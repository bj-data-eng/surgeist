use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlControlFacts, HtmlControlFactsLimits,
    HtmlCustomControlClassification, HtmlFormAssociation, HtmlFormAssociationState,
    HtmlSemanticCancellation, HtmlSemanticContext, HtmlSemanticContextInput,
    HtmlSemanticDependencies, HtmlSemanticFact, HtmlSemanticFailureKind, HtmlSemanticLimits,
    HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport, HtmlSemanticRequest,
    HtmlSemanticResource, HtmlSemanticUnsupported, HtmlSemanticValue,
};
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

fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::LabelFormOwner)
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
fn known(r: &HtmlSemanticReport) -> Option<&HtmlCaptureSubjectId> {
    match r.results()[0].fact() {
        HtmlSemanticFact::LabelFormOwner(HtmlSemanticValue::Known(v)) => v.as_ref(),
        other => panic!("expected known placeholder, got {other:?}"),
    }
}
fn missing(r: &HtmlSemanticReport) -> &HtmlSemanticMissing {
    match r.results()[0].fact() {
        HtmlSemanticFact::LabelFormOwner(HtmlSemanticValue::Missing(v)) => v,
        other => panic!("expected missing placeholder, got {other:?}"),
    }
}

fn controls_context(
    c: &HtmlCapture,
    facts: &[(HtmlCaptureSubjectId, HtmlFormAssociationState)],
    custom: &[(HtmlCaptureSubjectId, bool)],
) -> HtmlSemanticContext {
    let controls = HtmlControlFacts::new(
        c,
        c.host_facts_revision().clone(),
        facts
            .iter()
            .map(|(id, state)| HtmlFormAssociation::new(id.clone(), state.clone()))
            .collect(),
        vec![],
        vec![],
        vec![],
        HtmlControlFactsLimits::new(1000, 1000000),
    )
    .unwrap();
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(
            custom
                .iter()
                .map(|(id, v)| HtmlCustomControlClassification::new(id.clone(), *v))
                .collect(),
        )
        .with_controls(controls),
        1000,
    )
    .unwrap()
}
fn flat(
    kind: &str,
    label_attrs: &[(&str, &str)],
    control_attrs: &[(&str, &str)],
) -> (
    HtmlCapture,
    HtmlCaptureSubjectId,
    HtmlCaptureSubjectId,
    HtmlCaptureSubjectId,
) {
    let (mut b, d) = builder();
    let l = html(&mut b, &d, "label", label_attrs);
    let t = html(&mut b, &d, kind, control_attrs);
    let f = html(&mut b, &d, "form", &[("id", "f")]);
    for h in [&l, &t, &f] {
        children(&mut b, h, &[]);
    }
    children(&mut b, &d, &[l.clone(), t.clone(), f.clone()]);
    let c = b.finish().unwrap();
    let lid = c.subject_id(&l).unwrap();
    let tid = c.subject_id(&t).unwrap();
    let fid = c.subject_id(&f).unwrap();
    (c, lid, tid, fid)
}

#[test]
fn complete_no_control_and_empty_for_prove_null_without_association() {
    for attrs in [vec![], vec![("for", "")], vec![("for", "absent")]] {
        let (c, l, _, _) = flat("button", &attrs, &[("id", "x")]);
        assert_eq!(known(&report(&c, &context(&c), &l, 100)), None);
    }
    let (mut b, d) = builder();
    let l = html(
        &mut b,
        &d,
        "label",
        &[("for", ""), ("form", "irrelevant enormous value")],
    );
    let c = b.finish().unwrap();
    assert_eq!(
        known(&report(&c, &context(&c), &c.subject_id(&l).unwrap(), 0)),
        None
    );
}

#[test]
fn meter_and_progress_are_labelable_but_need_no_form_association_or_unused_form_values() {
    for kind in ["meter", "progress"] {
        let (c, l, t, _) = flat(
            kind,
            &[("for", "x"), ("form", "huge unused label form")],
            &[("id", "x"), ("form", "huge unused control form")],
        );
        let r = report(&c, &context(&c), &l, 1);
        assert_eq!(known(&r), None);
        let r = session(
            &c,
            &context(&c),
            limits(),
            1,
            HtmlSemanticCancellation::new(),
        )
        .query(&[HtmlSemanticRequest::new(t, HtmlSemanticQuery::FormOwner)])
        .unwrap();
        assert_eq!(
            r.results()[0].fact(),
            &HtmlSemanticFact::FormOwner(HtmlSemanticValue::Inapplicable)
        );
    }
}

#[test]
fn actual_nonancestor_control_owner_wins_over_label_ancestor_and_label_form_spelling() {
    let (mut b, d) = builder();
    let ancestor = html(&mut b, &d, "form", &[("id", "ancestor")]);
    let actual = html(&mut b, &d, "form", &[("id", "actual")]);
    let l = html(
        &mut b,
        &d,
        "label",
        &[("for", "x"), ("form", "wrong enormous value")],
    );
    let t = html(
        &mut b,
        &d,
        "button",
        &[("id", "x"), ("form", "also unused")],
    );
    children(&mut b, &ancestor, std::slice::from_ref(&l));
    children(&mut b, &actual, &[]);
    children(&mut b, &t, &[]);
    children(&mut b, &l, &[]);
    children(&mut b, &d, &[ancestor, t.clone(), actual.clone()]);
    let c = b.finish().unwrap();
    let lid = c.subject_id(&l).unwrap();
    let fid = c.subject_id(&actual).unwrap();
    let cx = controls_context(
        &c,
        &[(
            c.subject_id(&t).unwrap(),
            HtmlFormAssociationState::EstablishedCurrent(Some(fid.clone())),
        )],
        &[],
    );
    assert_eq!(known(&report(&c, &cx, &lid, 1)), Some(&fid));
    let r = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
        .query(&[HtmlSemanticRequest::new(lid, HtmlSemanticQuery::FormOwner)])
        .unwrap();
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::FormOwner(HtmlSemanticValue::Inapplicable)
    );
}

#[test]
fn established_null_differs_from_unavailable_and_absent_association() {
    let (c, l, t, _) = flat("button", &[("for", "x")], &[("id", "x")]);
    let cx = controls_context(
        &c,
        &[(
            t.clone(),
            HtmlFormAssociationState::EstablishedCurrent(None),
        )],
        &[],
    );
    assert_eq!(known(&report(&c, &cx, &l, 1)), None);
    let cx = controls_context(
        &c,
        &[(t.clone(), HtmlFormAssociationState::Unavailable)],
        &[],
    );
    assert_eq!(
        missing(&report(&c, &cx, &l, 1)),
        &HtmlSemanticMissing::FormAssociation(t.clone())
    );
    assert_eq!(
        missing(&report(&c, &context(&c), &l, 1)),
        &HtmlSemanticMissing::FormAssociation(t)
    );
}

#[test]
fn first_equal_id_nonlabelable_or_hidden_control_blocks_a_later_labelable_match() {
    for kind in ["div", "input"] {
        let (mut b, d) = builder();
        let l = html(&mut b, &d, "label", &[("for", "x")]);
        let first = html(&mut b, &d, kind, &[("id", "x"), ("type", "hidden")]);
        let later = html(&mut b, &d, "button", &[("id", "x")]);
        children(&mut b, &l, &[]);
        children(&mut b, &first, &[]);
        children(&mut b, &later, &[]);
        children(&mut b, &d, &[l.clone(), first, later]);
        let c = b.finish().unwrap();
        assert_eq!(
            known(&report(&c, &context(&c), &c.subject_id(&l).unwrap(), 6)),
            None
        );
    }
}

#[test]
fn implicit_first_labelable_control_uses_current_owner_and_ignores_later_unknown_children() {
    let (mut b, d) = builder();
    let l = html(&mut b, &d, "label", &[("form", "unused")]);
    let t = html(&mut b, &d, "textarea", &[]);
    let later = html(&mut b, &d, "div", &[]);
    let f = html(&mut b, &d, "form", &[]);
    children(&mut b, &l, &[t.clone(), later]);
    children(&mut b, &f, &[]);
    let c = b.finish().unwrap();
    let fid = c.subject_id(&f).unwrap();
    let cx = controls_context(
        &c,
        &[(
            c.subject_id(&t).unwrap(),
            HtmlFormAssociationState::EstablishedCurrent(Some(fid.clone())),
        )],
        &[],
    );
    assert_eq!(
        known(&report(&c, &cx, &c.subject_id(&l).unwrap(), 0)),
        Some(&fid)
    );
}

#[test]
fn reset_uses_control_form_attribute_and_first_form_id_without_label_fallback() {
    for blocked in [false, true] {
        let (mut b, d) = builder();
        let l = html(&mut b, &d, "label", &[("for", "x"), ("form", "ignored")]);
        let t = html(&mut b, &d, "button", &[("id", "x"), ("form", "f")]);
        let blocker = html(
            &mut b,
            &d,
            if blocked { "div" } else { "form" },
            &[("id", "f")],
        );
        let later = html(&mut b, &d, "form", &[("id", "f")]);
        for h in [&l, &t, &blocker, &later] {
            children(&mut b, h, &[]);
        }
        children(&mut b, &d, &[l.clone(), t.clone(), blocker.clone(), later]);
        let c = b.finish().unwrap();
        let cx = controls_context(
            &c,
            &[(
                c.subject_id(&t).unwrap(),
                HtmlFormAssociationState::ResetAgainstCapture,
            )],
            &[],
        );
        let r = report(&c, &cx, &c.subject_id(&l).unwrap(), 1);
        let blocker_id = c.subject_id(&blocker).unwrap();
        assert_eq!(known(&r), if blocked { None } else { Some(&blocker_id) });
    }
}

#[test]
fn reset_without_form_attribute_uses_control_ordinary_parent_not_label_ancestor() {
    let (mut b, d) = builder();
    let f = html(&mut b, &d, "form", &[]);
    let l = html(&mut b, &d, "label", &[]);
    let t = html(&mut b, &d, "button", &[]);
    children(&mut b, &l, std::slice::from_ref(&t));
    children(&mut b, &f, std::slice::from_ref(&l));
    let c = b.finish().unwrap();
    let cx = controls_context(
        &c,
        &[(
            c.subject_id(&t).unwrap(),
            HtmlFormAssociationState::ResetAgainstCapture,
        )],
        &[],
    );
    assert_eq!(
        known(&report(&c, &cx, &c.subject_id(&l).unwrap(), 0)),
        Some(&c.subject_id(&f).unwrap())
    );
}

#[test]
fn custom_classification_is_required_only_for_influencing_control() {
    let (c, l, t, f) = flat("x-control", &[("for", "x")], &[("id", "x")]);
    assert_eq!(
        missing(&report(&c, &context(&c), &l, 1)),
        &HtmlSemanticMissing::CustomControlClassification(t.clone())
    );
    let cx = controls_context(&c, &[], &[(t.clone(), false)]);
    assert_eq!(known(&report(&c, &cx, &l, 1)), None);
    let cx = controls_context(
        &c,
        &[(
            t.clone(),
            HtmlFormAssociationState::EstablishedCurrent(Some(f.clone())),
        )],
        &[(t, true)],
    );
    assert_eq!(known(&report(&c, &cx, &l, 1)), Some(&f));
}

#[test]
fn unavailable_target_topology_remains_missing_and_empty_for_bypasses_it() {
    let (mut b, d) = builder();
    let l = html(&mut b, &d, "label", &[]);
    let c = b.finish().unwrap();
    let lid = c.subject_id(&l).unwrap();
    assert_eq!(
        missing(&report(&c, &context(&c), &lid, 0)),
        &HtmlSemanticMissing::OrderedChildren(lid)
    );
    let (mut b, d) = builder();
    let l = html(&mut b, &d, "label", &[("for", "x")]);
    let c = b.finish().unwrap();
    let lid = c.subject_id(&l).unwrap();
    assert_eq!(
        missing(&report(&c, &context(&c), &lid, 1)),
        &HtmlSemanticMissing::Parent(lid)
    );
}

#[test]
fn case_namespace_and_non_element_roles_preserve_typed_applicability() {
    let (mut b, d) = builder();
    let upper = html(&mut b, &d, "LABEL", &[]);
    let foreign = element(
        &mut b,
        &d,
        Namespace::svg(),
        "label",
        OrderedAttributes::new([]).unwrap(),
    );
    let text = b.add_text(&d, "text".into()).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    for h in [upper, text, d] {
        assert_eq!(
            report(&c, &cx, &c.subject_id(&h).unwrap(), 0).results()[0].fact(),
            &HtmlSemanticFact::LabelFormOwner(HtmlSemanticValue::Inapplicable)
        );
    }
    assert_eq!(
        report(&c, &cx, &c.subject_id(&foreign).unwrap(), 0).results()[0].fact(),
        &HtmlSemanticFact::LabelFormOwner(HtmlSemanticValue::Unsupported(
            HtmlSemanticUnsupported::ForeignNamespace
        ))
    );
}

#[test]
fn exact_utf16_for_id_matching_and_original_for_raw_cap_are_preserved() {
    let (mut b, d) = builder();
    let raw = DocumentString::from_utf16(vec![0xdfff, 49]);
    let l = element(
        &mut b,
        &d,
        Namespace::html(),
        "label",
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::none(), None, "for"),
            raw.clone(),
        )])
        .unwrap(),
    );
    let t = element(
        &mut b,
        &d,
        Namespace::html(),
        "button",
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::none(), None, "id"),
            raw,
        )])
        .unwrap(),
    );
    let f = html(&mut b, &d, "form", &[]);
    for h in [&l, &t, &f] {
        children(&mut b, h, &[]);
    }
    children(&mut b, &d, &[l.clone(), t.clone(), f.clone()]);
    let c = b.finish().unwrap();
    let lid = c.subject_id(&l).unwrap();
    let fid = c.subject_id(&f).unwrap();
    let cx = controls_context(
        &c,
        &[(
            c.subject_id(&t).unwrap(),
            HtmlFormAssociationState::EstablishedCurrent(Some(fid.clone())),
        )],
        &[],
    );
    assert_eq!(known(&report(&c, &cx, &lid, 2)), Some(&fid));
    let failure = session(&c, &cx, limits(), 1, HtmlSemanticCancellation::new())
        .query(&[request(&lid)])
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
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
        0,
    )
}
#[test]
fn actual_lookup_work_scratch_bytes_and_output_have_exact_one_short_sticky_boundaries() {
    let (c, l, t, f) = flat("button", &[("for", "x")], &[("id", "x"), ("form", "f")]);
    let cx = controls_context(
        &c,
        &[(t, HtmlFormAssociationState::ResetAgainstCapture)],
        &[],
    );
    let u = report(&c, &cx, &l, 1).usage();
    assert_eq!(u.nested_output_records(), 0);
    for (resource, exact) in [
        (HtmlSemanticResource::Work, u.work()),
        (HtmlSemanticResource::OutputRecords, u.output_records()),
        (
            HtmlSemanticResource::ScratchRecords,
            u.peak_scratch_records(),
        ),
        (HtmlSemanticResource::AllocatedBytes, u.allocated_bytes()),
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
            let result = ss.query(&[request(&l)]);
            if cap == exact {
                assert_eq!(known(&result.unwrap()), Some(&f));
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
fn late_batch_raw_failure_and_cancellation_retain_original_qualification_and_no_prefix() {
    let (mut b, d) = builder();
    let a = html(&mut b, &d, "label", &[("for", "")]);
    let z = html(&mut b, &d, "label", &[("for", "long")]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let a = c.subject_id(&a).unwrap();
    let z = c.subject_id(&z).unwrap();
    let mut ss = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let failure = ss.query(&[request(&a), request(&z)]).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    assert_eq!(failure.qualification().unwrap().capture(), c.identity());
    assert_eq!(failure.qualification().unwrap().context(), cx.identity());
    assert_eq!(ss.query(&[]).unwrap_err(), failure);
    assert_eq!(ss.usage().scratch_records(), 0);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut ss = session(&c, &cx, limits(), 100, cancel);
    let failure = ss.query(&[request(&a)]).unwrap_err();
    assert_eq!(failure.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(ss.query(&[]).unwrap_err(), failure);
}

#[test]
fn state_only_actual_owner_changes_have_distinct_context_qualification() {
    let (c, l, t, f) = flat("button", &[("for", "x")], &[("id", "x")]);
    let a = controls_context(
        &c,
        &[(
            t.clone(),
            HtmlFormAssociationState::EstablishedCurrent(None),
        )],
        &[],
    );
    let b = controls_context(
        &c,
        &[(
            t,
            HtmlFormAssociationState::EstablishedCurrent(Some(f.clone())),
        )],
        &[],
    );
    let ar = report(&c, &a, &l, 1);
    let br = report(&c, &b, &l, 1);
    assert_eq!(known(&ar), None);
    assert_eq!(known(&br), Some(&f));
    assert_eq!(ar.capture(), br.capture());
    assert_ne!(ar.context(), br.context());
    assert_eq!(
        ar.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
    let (other, foreign, _, _) = flat("button", &[("for", "x")], &[("id", "x")]);
    let _ = other;
    let failure = session(&c, &a, limits(), 1, HtmlSemanticCancellation::new())
        .query(&[request(&foreign)])
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Capture(_)
    ));
}

#[test]
fn namespaced_for_does_not_override_implicit_control_and_namespace_id_is_not_a_match() {
    let (mut b, d) = builder();
    let l = element(
        &mut b,
        &d,
        Namespace::html(),
        "label",
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::svg(), None, "for"),
            "oversized ignored",
        )])
        .unwrap(),
    );
    let t = html(&mut b, &d, "button", &[]);
    let f = html(&mut b, &d, "form", &[]);
    children(&mut b, &l, std::slice::from_ref(&t));
    let c = b.finish().unwrap();
    let fid = c.subject_id(&f).unwrap();
    let cx = controls_context(
        &c,
        &[(
            c.subject_id(&t).unwrap(),
            HtmlFormAssociationState::EstablishedCurrent(Some(fid.clone())),
        )],
        &[],
    );
    assert_eq!(
        known(&report(&c, &cx, &c.subject_id(&l).unwrap(), 0)),
        Some(&fid)
    );
    let (mut b, d) = builder();
    let l = html(&mut b, &d, "label", &[("for", "x")]);
    let t = element(
        &mut b,
        &d,
        Namespace::html(),
        "button",
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::svg(), None, "id"),
            "x",
        )])
        .unwrap(),
    );
    children(&mut b, &l, &[]);
    children(&mut b, &t, &[]);
    children(&mut b, &d, &[l.clone(), t]);
    let c = b.finish().unwrap();
    assert_eq!(
        known(&report(&c, &context(&c), &c.subject_id(&l).unwrap(), 1)),
        None
    );
}

#[test]
fn unknown_earlier_order_is_missing_but_resolved_first_id_ignores_later_unknown_subtree() {
    for prefix in [true, false] {
        let (mut b, d) = builder();
        let l = html(&mut b, &d, "label", &[("for", "x")]);
        let t = html(&mut b, &d, "button", &[("id", "x")]);
        let w = html(&mut b, &d, "div", &[]);
        let f = html(&mut b, &d, "form", &[]);
        children(&mut b, &l, &[]);
        children(&mut b, &t, &[]);
        children(&mut b, &f, &[]);
        let order = if prefix {
            vec![l.clone(), w.clone(), t.clone(), f.clone()]
        } else {
            vec![l.clone(), t.clone(), w.clone(), f.clone()]
        };
        children(&mut b, &d, &order);
        let c = b.finish().unwrap();
        let fid = c.subject_id(&f).unwrap();
        let cx = controls_context(
            &c,
            &[(
                c.subject_id(&t).unwrap(),
                HtmlFormAssociationState::EstablishedCurrent(Some(fid.clone())),
            )],
            &[],
        );
        let r = report(&c, &cx, &c.subject_id(&l).unwrap(), 1);
        if prefix {
            assert_eq!(
                missing(&r),
                &HtmlSemanticMissing::OrderedChildren(c.subject_id(&w).unwrap())
            );
        } else {
            assert_eq!(known(&r), Some(&fid));
        }
        assert_eq!(r.usage().scratch_records(), 0);
    }
}
