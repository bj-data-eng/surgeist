use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlCapturedShadowProperties, HtmlControlFacts,
    HtmlControlFactsLimits, HtmlFormAssociation, HtmlFormAssociationState, HtmlFormOwnerOrigin,
    HtmlOptionFormOwner, HtmlSemanticCancellation, HtmlSemanticContext, HtmlSemanticContextInput,
    HtmlSemanticDependencies, HtmlSemanticFact, HtmlSemanticFailureKind, HtmlSemanticLimits,
    HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport, HtmlSemanticRequest,
    HtmlSemanticResource, HtmlSemanticUnsupported, HtmlSemanticValue,
};
#[path = "common/builder.rs"]
mod shared_builder;
use shared_builder::builder;
#[path = "common/session.rs"]
mod shared_session;
use shared_session::session;
use surgeist_html::*;

fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    ns: Namespace,
    local: &str,
    attrs: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(ns, None, local),
        OrderedAttributes::new(
            attrs
                .iter()
                .map(|(n, v)| Attribute::new(QualifiedName::new(Namespace::none(), None, *n), *v)),
        )
        .unwrap(),
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
    element(b, d, Namespace::html(), local, attrs)
}
fn children(
    b: &mut HtmlCaptureBuilder,
    p: &HtmlCaptureBuilderSubject,
    cs: &[HtmlCaptureBuilderSubject],
) {
    b.establish_children(p, cs).unwrap();
}
fn context(
    c: &HtmlCapture,
    associations: &[(HtmlCaptureSubjectId, HtmlFormAssociationState)],
) -> HtmlSemanticContext {
    let controls = HtmlControlFacts::new(
        c,
        c.host_facts_revision().clone(),
        associations
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
        HtmlSemanticContextInput::new(vec![]).with_controls(controls),
        0,
    )
    .unwrap()
}
fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20000000, 100, 1000, 0, 1000000, 1000)
}

fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::OptionFormOwner)
}
fn report(
    c: &HtmlCapture,
    cx: &HtmlSemanticContext,
    id: &HtmlCaptureSubjectId,
) -> HtmlSemanticReport {
    session(c, cx, limits(), 100000, HtmlSemanticCancellation::new())
        .query(&[request(id)])
        .unwrap()
}
fn observation(r: &HtmlSemanticReport) -> &HtmlOptionFormOwner {
    match r.results()[0].fact() {
        HtmlSemanticFact::OptionFormOwner(HtmlSemanticValue::Known(v)) => v,
        other => panic!("expected known option form observation, got {other:?}"),
    }
}
fn missing(r: &HtmlSemanticReport) -> &HtmlSemanticMissing {
    match r.results()[0].fact() {
        HtmlSemanticFact::OptionFormOwner(HtmlSemanticValue::Missing(v)) => v,
        other => panic!("expected missing, got {other:?}"),
    }
}

#[test]
fn complete_detached_root_proves_no_select_without_association_children_or_raw_values() {
    let (mut b, d) = builder();
    let o = html(
        &mut b,
        &d,
        "option",
        &[
            ("form", "unused enormous form"),
            ("label", "unused enormous label"),
            ("value", "unused enormous value"),
            ("selected", ""),
        ],
    );
    b.establish_root(&o).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    let id = c.subject_id(&o).unwrap();
    let r = session(
        &c,
        &cx,
        HtmlSemanticLimits::new(limits().work(), 1, 0, 0, 100000, 0),
        0,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(&id)])
    .unwrap();
    assert_eq!(observation(&r).select(), None);
    assert_eq!(observation(&r).form_owner(), None);
    assert_eq!(r.usage().peak_scratch_records(), 0);
    assert_eq!(r.usage().nested_output_records(), 0);
    // Generic FormOwner remains restricted to form-associated elements.
    let r = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
        .query(&[HtmlSemanticRequest::new(id, HtmlSemanticQuery::FormOwner)])
        .unwrap();
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::FormOwner(HtmlSemanticValue::Inapplicable)
    );
}

#[test]
fn unavailable_parent_is_missing_not_a_fabricated_null_select() {
    let (mut b, d) = builder();
    let o = html(&mut b, &d, "option", &[]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&o).unwrap();
    let r = report(&c, &context(&c, &[]), &id);
    assert_eq!(missing(&r), &HtmlSemanticMissing::Parent(id));
}

#[test]
fn each_source_barrier_proves_null_without_unknown_ancestry_above_it() {
    for barrier in ["datalist", "hr", "option", "second-optgroup"] {
        let (mut b, d) = builder();
        let o = html(&mut b, &d, "option", &[]);
        let p = html(
            &mut b,
            &d,
            if barrier == "second-optgroup" {
                "optgroup"
            } else {
                barrier
            },
            &[],
        );
        children(&mut b, &p, std::slice::from_ref(&o));
        if barrier == "second-optgroup" {
            let outer = html(&mut b, &d, "optgroup", &[]);
            children(&mut b, &outer, &[p]);
        }
        let c = b.finish().unwrap();
        let r = report(&c, &context(&c, &[]), &c.subject_id(&o).unwrap());
        assert_eq!(observation(&r).select(), None);
        assert_eq!(observation(&r).form_owner(), None);
    }
}

#[test]
fn one_optgroup_and_foreign_or_uppercase_barrier_lookalikes_reach_select() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[]);
    let o = html(&mut b, &d, "option", &[]);
    let mut next = o.clone();
    for (ns, name) in [
        (Namespace::html(), "optgroup"),
        (Namespace::svg(), "option"),
        (Namespace::svg(), "optgroup"),
        (Namespace::html(), "DATALIST"),
        (Namespace::html(), "HR"),
    ] {
        let w = element(&mut b, &d, ns, name, &[]);
        children(&mut b, &w, &[next]);
        next = w;
    }
    children(&mut b, &s, &[next]);
    let c = b.finish().unwrap();
    let sid = c.subject_id(&s).unwrap();
    let cx = context(
        &c,
        &[(
            sid.clone(),
            HtmlFormAssociationState::EstablishedCurrent(None),
        )],
    );
    let r = report(&c, &cx, &c.subject_id(&o).unwrap());
    assert_eq!(observation(&r).select(), Some(&sid));
    let owner = observation(&r).form_owner().unwrap();
    assert_eq!(owner.owner(), None);
    assert_eq!(owner.origin(), HtmlFormOwnerOrigin::EstablishedCurrent);
}

#[test]
fn nearest_select_wins_over_outer_select_and_nearer_form_ancestor() {
    let (mut b, d) = builder();
    let outer = html(&mut b, &d, "select", &[]);
    let inner = html(&mut b, &d, "select", &[]);
    let form = html(&mut b, &d, "form", &[]);
    let o = html(&mut b, &d, "option", &[]);
    children(&mut b, &form, std::slice::from_ref(&o));
    children(&mut b, &inner, &[form]);
    children(&mut b, &outer, std::slice::from_ref(&inner));
    let c = b.finish().unwrap();
    let iid = c.subject_id(&inner).unwrap();
    let cx = context(
        &c,
        &[(
            iid.clone(),
            HtmlFormAssociationState::EstablishedCurrent(None),
        )],
    );
    let r = report(&c, &cx, &c.subject_id(&o).unwrap());
    assert_eq!(observation(&r).select(), Some(&iid));
    assert_eq!(observation(&r).form_owner().unwrap().owner(), None);
}

#[test]
fn actual_nonancestor_owner_and_actual_null_preserve_origin_with_unavailable_select_parent() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[]);
    let form = html(&mut b, &d, "form", &[]);
    let o = html(
        &mut b,
        &d,
        "option",
        &[("value", "unused"), ("selected", "")],
    );
    children(&mut b, &s, std::slice::from_ref(&o));
    let c = b.finish().unwrap();
    let sid = c.subject_id(&s).unwrap();
    let fid = c.subject_id(&form).unwrap();
    for owner in [Some(fid.clone()), None] {
        let cx = context(
            &c,
            &[(
                sid.clone(),
                HtmlFormAssociationState::EstablishedCurrent(owner.clone()),
            )],
        );
        let r = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
            .query(&[request(&c.subject_id(&o).unwrap())])
            .unwrap();
        assert_eq!(observation(&r).select(), Some(&sid));
        assert_eq!(
            observation(&r).form_owner().unwrap().owner(),
            owner.as_ref()
        );
        assert_eq!(
            observation(&r).form_owner().unwrap().origin(),
            HtmlFormOwnerOrigin::EstablishedCurrent
        );
    }
}

#[test]
fn absent_and_explicit_unavailable_select_association_are_missing_on_select() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[]);
    let o = html(&mut b, &d, "option", &[]);
    children(&mut b, &s, std::slice::from_ref(&o));
    let c = b.finish().unwrap();
    let sid = c.subject_id(&s).unwrap();
    for associations in [
        vec![],
        vec![(sid.clone(), HtmlFormAssociationState::Unavailable)],
    ] {
        let cx = context(&c, &associations);
        assert_eq!(
            missing(&report(&c, &cx, &c.subject_id(&o).unwrap())),
            &HtmlSemanticMissing::FormAssociation(sid.clone())
        );
    }
}

#[test]
fn reset_qualified_select_resolves_nearest_form_and_root_null_with_reset_origin() {
    for enclosed in [false, true] {
        let (mut b, d) = builder();
        let s = html(&mut b, &d, "select", &[]);
        let o = html(&mut b, &d, "option", &[]);
        children(&mut b, &s, std::slice::from_ref(&o));
        let form = if enclosed {
            let f = html(&mut b, &d, "form", &[]);
            children(&mut b, &f, std::slice::from_ref(&s));
            b.establish_root(&f).unwrap();
            Some(f)
        } else {
            b.establish_root(&s).unwrap();
            None
        };
        let c = b.finish().unwrap();
        let sid = c.subject_id(&s).unwrap();
        let cx = context(
            &c,
            &[(sid.clone(), HtmlFormAssociationState::ResetAgainstCapture)],
        );
        let r = report(&c, &cx, &c.subject_id(&o).unwrap());
        assert_eq!(observation(&r).select(), Some(&sid));
        assert_eq!(
            observation(&r).form_owner().unwrap().owner(),
            form.as_ref().map(|f| c.subject_id(f).unwrap()).as_ref()
        );
        assert_eq!(
            observation(&r).form_owner().unwrap().origin(),
            HtmlFormOwnerOrigin::ResetAgainstCapture
        );
    }
}

fn external_reset(
    nonform_first: bool,
    unknown_first: bool,
) -> (
    HtmlCapture,
    HtmlCaptureSubjectId,
    HtmlCaptureSubjectId,
    HtmlCaptureSubjectId,
    Option<HtmlCaptureSubjectId>,
) {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("form", "target")]);
    let o = html(&mut b, &d, "option", &[("form", "not consulted")]);
    let form = html(&mut b, &d, "form", &[("id", "target")]);
    children(&mut b, &s, std::slice::from_ref(&o));
    children(&mut b, &form, &[]);
    let prefix = if nonform_first || unknown_first {
        let p = html(
            &mut b,
            &d,
            "div",
            if nonform_first {
                &[("id", "target")]
            } else {
                &[]
            },
        );
        if !unknown_first {
            children(&mut b, &p, &[]);
        }
        Some(p)
    } else {
        None
    };
    let mut order: Vec<_> = prefix.iter().cloned().collect();
    order.extend([form.clone(), s.clone()]);
    children(&mut b, &d, &order);
    let c = b.finish().unwrap();
    let ids = (
        c.subject_id(&o).unwrap(),
        c.subject_id(&s).unwrap(),
        c.subject_id(&form).unwrap(),
        prefix.as_ref().map(|p| c.subject_id(p).unwrap()),
    );
    (c, ids.0, ids.1, ids.2, ids.3)
}

#[test]
fn reset_external_form_uses_first_id_before_type_and_retains_original_raw_cap() {
    for blocked in [false, true] {
        let (c, o, s, form, _) = external_reset(blocked, false);
        let cx = context(
            &c,
            &[(s.clone(), HtmlFormAssociationState::ResetAgainstCapture)],
        );
        let r = session(&c, &cx, limits(), 6, HtmlSemanticCancellation::new())
            .query(&[request(&o)])
            .unwrap();
        assert_eq!(observation(&r).select(), Some(&s));
        let owner = observation(&r).form_owner().unwrap();
        assert_eq!(owner.origin(), HtmlFormOwnerOrigin::ResetAgainstCapture);
        assert_eq!(owner.owner(), if blocked { None } else { Some(&form) });
        let failure = session(&c, &cx, limits(), 5, HtmlSemanticCancellation::new())
            .query(&[request(&o)])
            .unwrap_err();
        assert!(matches!(
            failure.kind(),
            HtmlSemanticFailureKind::Attribute(_)
        ));
    }
}

#[test]
fn reset_influencing_parent_and_earlier_lookup_coverage_remain_missing() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[]);
    let o = html(&mut b, &d, "option", &[]);
    children(&mut b, &s, std::slice::from_ref(&o));
    let c = b.finish().unwrap();
    let sid = c.subject_id(&s).unwrap();
    let cx = context(
        &c,
        &[(sid.clone(), HtmlFormAssociationState::ResetAgainstCapture)],
    );
    assert_eq!(
        missing(&report(&c, &cx, &c.subject_id(&o).unwrap())),
        &HtmlSemanticMissing::Parent(sid)
    );
    let (c, o, s, _, prefix) = external_reset(false, true);
    let cx = context(&c, &[(s, HtmlFormAssociationState::ResetAgainstCapture)]);
    let r = report(&c, &cx, &o);
    assert_eq!(
        missing(&r),
        &HtmlSemanticMissing::OrderedChildren(prefix.unwrap())
    );
    assert_eq!(r.usage().scratch_records(), 0);
}

#[test]
fn shadow_and_template_fragments_do_not_substitute_host_select() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let host = html(&mut b, &d, if shadow { "select" } else { "template" }, &[]);
        let fragment = if shadow {
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
        let o = html(&mut b, &d, "option", &[]);
        children(&mut b, &fragment, std::slice::from_ref(&o));
        let c = b.finish().unwrap();
        let r = report(&c, &context(&c, &[]), &c.subject_id(&o).unwrap());
        assert_eq!(observation(&r).select(), None);
        assert_eq!(observation(&r).form_owner(), None);
    }
}

#[test]
fn exact_subject_applicability_does_not_require_association_or_parent_facts() {
    let (mut b, d) = builder();
    let upper = html(&mut b, &d, "OPTION", &[]);
    let s = html(&mut b, &d, "select", &[]);
    let foreign = element(&mut b, &d, Namespace::svg(), "option", &[]);
    let text = b.add_text(&d, "text".into()).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    for h in [upper, s, text, d] {
        assert_eq!(
            report(&c, &cx, &c.subject_id(&h).unwrap()).results()[0].fact(),
            &HtmlSemanticFact::OptionFormOwner(HtmlSemanticValue::Inapplicable)
        );
    }
    assert_eq!(
        report(&c, &cx, &c.subject_id(&foreign).unwrap()).results()[0].fact(),
        &HtmlSemanticFact::OptionFormOwner(HtmlSemanticValue::Unsupported(
            HtmlSemanticUnsupported::ForeignNamespace
        ))
    );
}

#[test]
fn state_only_owner_changes_recompute_with_exact_capture_context_and_host_keys() {
    let (c, o, s, form, _) = external_reset(false, false);
    let a = context(
        &c,
        &[(
            s.clone(),
            HtmlFormAssociationState::EstablishedCurrent(None),
        )],
    );
    let b = context(
        &c,
        &[(
            s,
            HtmlFormAssociationState::EstablishedCurrent(Some(form.clone())),
        )],
    );
    let ar = report(&c, &a, &o);
    let br = report(&c, &b, &o);
    assert_eq!(observation(&ar).form_owner().unwrap().owner(), None);
    assert_eq!(observation(&br).form_owner().unwrap().owner(), Some(&form));
    assert_eq!(ar.capture(), br.capture());
    assert_ne!(ar.context(), br.context());
    assert_eq!(ar.host_revision(), c.host_facts_revision());
    assert_eq!(
        ar.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
}

#[test]
fn exact_and_one_short_work_output_reset_scratch_and_bytes_limits_are_sticky() {
    let (c, o, s, _, _) = external_reset(false, false);
    let cx = context(&c, &[(s, HtmlFormAssociationState::ResetAgainstCapture)]);
    let req = request(&o);
    let baseline = report(&c, &cx, &o).usage();
    assert_eq!(baseline.nested_output_records(), 0);
    for (resource, exact) in [
        (HtmlSemanticResource::Work, baseline.work()),
        (
            HtmlSemanticResource::OutputRecords,
            baseline.output_records(),
        ),
        (
            HtmlSemanticResource::ScratchRecords,
            baseline.peak_scratch_records(),
        ),
        (
            HtmlSemanticResource::AllocatedBytes,
            baseline.allocated_bytes(),
        ),
    ] {
        assert!(exact > 0);
        for cap in [exact, exact - 1] {
            let l = HtmlSemanticLimits::new(
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
            );
            let mut ss = session(&c, &cx, l, 6, HtmlSemanticCancellation::new());
            let r = ss.query(std::slice::from_ref(&req));
            if cap == exact {
                assert!(r.is_ok(), "{resource:?}: {r:?}");
            } else {
                let f = r.unwrap_err();
                assert!(
                    matches!(f.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r==resource),
                    "{resource:?}: {f:?}"
                );
                assert_eq!(f.qualification().unwrap().context(), cx.identity());
                assert_eq!(ss.query(&[]).unwrap_err(), f);
            }
            assert_eq!(ss.usage().scratch_records(), 0);
        }
    }
}

#[test]
fn late_batch_failure_discards_prefix_and_cancellation_foreign_capture_stay_qualified() {
    let (c, o, s, _, _) = external_reset(false, false);
    let cx = context(&c, &[(s, HtmlFormAssociationState::ResetAgainstCapture)]);
    let mut ss = session(
        &c,
        &cx,
        HtmlSemanticLimits::new(limits().work(), 1, 1000, 0, 1000000, 0),
        6,
        HtmlSemanticCancellation::new(),
    );
    let fail = ss.query(&[request(&o), request(&o)]).unwrap_err();
    assert!(matches!(
        fail.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::OutputRecords,
            ..
        }
    ));
    assert_eq!(fail.qualification().unwrap().capture(), c.identity());
    assert_eq!(ss.query(&[]).unwrap_err(), fail);
    assert_eq!(ss.usage().scratch_records(), 0);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut ss = session(&c, &cx, limits(), 6, cancel);
    let fail = ss.query(&[request(&o)]).unwrap_err();
    assert_eq!(fail.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(ss.query(&[]).unwrap_err(), fail);
    let (_, other, _, _, _) = external_reset(false, false);
    let fail = session(&c, &cx, limits(), 6, HtmlSemanticCancellation::new())
        .query(&[request(&other)])
        .unwrap_err();
    assert!(matches!(fail.kind(), HtmlSemanticFailureKind::Capture(_)));
}
