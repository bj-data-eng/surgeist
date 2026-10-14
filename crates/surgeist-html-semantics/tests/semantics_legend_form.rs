use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlCapturedShadowProperties, HtmlControlFacts,
    HtmlControlFactsLimits, HtmlFormAssociation, HtmlFormAssociationState, HtmlFormOwnerOrigin,
    HtmlSemanticCancellation, HtmlSemanticContext, HtmlSemanticContextInput,
    HtmlSemanticDependencies, HtmlSemanticFact, HtmlSemanticFailureKind, HtmlSemanticLimits,
    HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport, HtmlSemanticRequest,
    HtmlSemanticResource, HtmlSemanticResult, HtmlSemanticUnsupported, HtmlSemanticValue,
};
#[path = "common/builder.rs"]
mod shared_builder;
use shared_builder::builder;
#[path = "common/session.rs"]
mod shared_session;
use shared_session::session;
use surgeist_html::*;

// Literal expectations follow HTMLLegendElement.form: immediate fieldset
// parent -> that fieldset's form getter; every other established parent -> null.

fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    name: QualifiedName,
    attrs: OrderedAttributes,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        name,
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
        QualifiedName::new(Namespace::html(), None, local),
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
    parent: &HtmlCaptureBuilderSubject,
    nodes: &[HtmlCaptureBuilderSubject],
) {
    b.establish_children(parent, nodes).unwrap();
}
fn context(
    c: &HtmlCapture,
    facts: &[(HtmlCaptureSubjectId, HtmlFormAssociationState)],
) -> HtmlSemanticContext {
    let input = if facts.is_empty() {
        HtmlSemanticContextInput::new(vec![])
    } else {
        HtmlSemanticContextInput::new(vec![]).with_controls(
            HtmlControlFacts::new(
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
            .unwrap(),
        )
    };
    HtmlSemanticContext::new(c, c.host_facts_revision().clone(), input, 0).unwrap()
}
fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20000000, 100, 1000, 0, 1000000, 0)
}

fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::LegendFormOwner)
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
fn value(r: &HtmlSemanticReport, index: usize) -> &HtmlSemanticValue<Option<HtmlCaptureSubjectId>> {
    match r.results()[index].fact() {
        HtmlSemanticFact::LegendFormOwner(v) => v,
        other => panic!("wrong getter fact: {other:?}"),
    }
}
fn direct() -> (
    HtmlCapture,
    HtmlCaptureSubjectId,
    HtmlCaptureSubjectId,
    HtmlCaptureSubjectId,
) {
    let (mut b, d) = builder();
    let fs = html(&mut b, &d, "fieldset", &[]);
    let legend = html(&mut b, &d, "legend", &[]);
    let form = html(&mut b, &d, "form", &[]);
    children(&mut b, &fs, std::slice::from_ref(&legend));
    children(&mut b, &d, &[fs.clone(), form.clone()]);
    let c = b.finish().unwrap();
    let ids = (
        c.subject_id(&legend).unwrap(),
        c.subject_id(&fs).unwrap(),
        c.subject_id(&form).unwrap(),
    );
    (c, ids.0, ids.1, ids.2)
}

#[test]
fn actual_nonancestor_fieldset_owner_wins_over_ancestor_and_legend_form_attribute() {
    let (mut b, d) = builder();
    let ancestor = html(&mut b, &d, "form", &[("id", "ancestor")]);
    let actual = html(&mut b, &d, "form", &[("id", "actual")]);
    let fs = html(&mut b, &d, "fieldset", &[("form", "unused huge spelling")]);
    let legend = html(
        &mut b,
        &d,
        "legend",
        &[("form", "ancestor"), ("title", "irrelevant")],
    );
    children(&mut b, &fs, std::slice::from_ref(&legend));
    children(&mut b, &ancestor, std::slice::from_ref(&fs));
    children(&mut b, &d, &[ancestor, actual.clone()]);
    let c = b.finish().unwrap();
    let lid = c.subject_id(&legend).unwrap();
    let fsid = c.subject_id(&fs).unwrap();
    let fid = c.subject_id(&actual).unwrap();
    let cx = context(
        &c,
        &[(
            fsid.clone(),
            HtmlFormAssociationState::EstablishedCurrent(Some(fid.clone())),
        )],
    );
    let r = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
        .query(&[
            request(&lid),
            HtmlSemanticRequest::new(fsid, HtmlSemanticQuery::FormOwner),
            HtmlSemanticRequest::new(lid, HtmlSemanticQuery::FormOwner),
        ])
        .unwrap();
    assert_eq!(value(&r, 0), &HtmlSemanticValue::Known(Some(fid.clone())));
    match r.results()[1].fact() {
        HtmlSemanticFact::FormOwner(HtmlSemanticValue::Known(owner)) => {
            assert_eq!(owner.owner(), Some(&fid));
            assert_eq!(owner.origin(), HtmlFormOwnerOrigin::EstablishedCurrent);
        }
        other => panic!("wrong fieldset owner: {other:?}"),
    }
    assert_eq!(
        r.results()[2].fact(),
        &HtmlSemanticFact::FormOwner(HtmlSemanticValue::Inapplicable)
    );
}

#[test]
fn established_null_is_distinct_from_absent_or_unavailable_fieldset_association() {
    let (c, legend, fs, _) = direct();
    let cx = context(
        &c,
        &[(
            fs.clone(),
            HtmlFormAssociationState::EstablishedCurrent(None),
        )],
    );
    assert_eq!(
        value(&report(&c, &cx, &legend, 0), 0),
        &HtmlSemanticValue::Known(None)
    );
    for facts in [
        vec![],
        vec![(fs.clone(), HtmlFormAssociationState::Unavailable)],
    ] {
        assert_eq!(
            value(&report(&c, &context(&c, &facts), &legend, 0), 0),
            &HtmlSemanticValue::Missing(HtmlSemanticMissing::FormAssociation(fs.clone()))
        );
    }
}

#[test]
fn every_direct_legend_observes_fieldset_owner_without_first_legend_or_child_order_rule() {
    let (mut b, d) = builder();
    let fs = html(&mut b, &d, "fieldset", &[("disabled", "oversized unread")]);
    let first = html(&mut b, &d, "legend", &[]);
    let second = html(&mut b, &d, "legend", &[]);
    let f = html(&mut b, &d, "form", &[]);
    // Establish only parent edges, with fieldset child order still unavailable.
    b.establish_parent(&first, &fs).unwrap();
    b.establish_parent(&second, &fs).unwrap();
    children(&mut b, &d, &[fs.clone(), f.clone()]);
    let c = b.finish().unwrap();
    let fid = c.subject_id(&f).unwrap();
    let cx = context(
        &c,
        &[(
            c.subject_id(&fs).unwrap(),
            HtmlFormAssociationState::EstablishedCurrent(Some(fid.clone())),
        )],
    );
    let r = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
        .query(&[
            request(&c.subject_id(&first).unwrap()),
            request(&c.subject_id(&second).unwrap()),
        ])
        .unwrap();
    for i in 0..2 {
        assert_eq!(value(&r, i), &HtmlSemanticValue::Known(Some(fid.clone())));
    }
}

#[test]
fn wrapper_optgroup_and_custom_parent_prove_null_without_association_or_ancestry() {
    for parent in ["div", "optgroup", "x-control", "FIELDSET"] {
        let (mut b, d) = builder();
        let p = html(&mut b, &d, parent, &[("form", "unread")]);
        let legend = html(&mut b, &d, "legend", &[]);
        b.establish_parent(&legend, &p).unwrap();
        let c = b.finish().unwrap();
        assert_eq!(
            value(
                &report(&c, &context(&c, &[]), &c.subject_id(&legend).unwrap(), 0),
                0
            ),
            &HtmlSemanticValue::Known(None)
        );
    }
    let (mut b, d) = builder();
    let fs = html(&mut b, &d, "fieldset", &[]);
    let wrapper = html(&mut b, &d, "div", &[]);
    let legend = html(&mut b, &d, "legend", &[]);
    children(&mut b, &fs, std::slice::from_ref(&wrapper));
    children(&mut b, &wrapper, std::slice::from_ref(&legend));
    let c = b.finish().unwrap();
    assert_eq!(
        value(
            &report(&c, &context(&c, &[]), &c.subject_id(&legend).unwrap(), 0),
            0
        ),
        &HtmlSemanticValue::Known(None)
    );
}

#[test]
fn established_root_document_or_fragment_parent_is_null_unavailable_parent_is_missing() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "legend", &[]);
    let doc_child = html(&mut b, &d, "legend", &[]);
    let frag_child = html(&mut b, &d, "legend", &[]);
    let missing = html(&mut b, &d, "legend", &[]);
    let fragment = b.add_fragment(&d).unwrap();
    b.establish_root(&root).unwrap();
    children(&mut b, &d, std::slice::from_ref(&doc_child));
    children(&mut b, &fragment, std::slice::from_ref(&frag_child));
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    for h in [&root, &doc_child, &frag_child] {
        assert_eq!(
            value(&report(&c, &cx, &c.subject_id(h).unwrap(), 0), 0),
            &HtmlSemanticValue::Known(None)
        );
    }
    let id = c.subject_id(&missing).unwrap();
    assert_eq!(
        value(&report(&c, &cx, &id, 0), 0),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(id))
    );
}

#[test]
fn hosted_roots_do_not_substitute_host_for_immediate_parent() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let host = html(&mut b, &d, if shadow { "div" } else { "template" }, &[]);
        let root = if shadow {
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
            let fragment = b.add_fragment(&d).unwrap();
            b.establish_template_contents(&host, &fragment).unwrap();
            fragment
        };
        let legend = html(&mut b, &d, "legend", &[]);
        children(&mut b, &root, std::slice::from_ref(&legend));
        let c = b.finish().unwrap();
        assert_eq!(
            value(
                &report(&c, &context(&c, &[]), &c.subject_id(&legend).unwrap(), 0),
                0
            ),
            &HtmlSemanticValue::Known(None)
        );
    }
}

#[test]
fn exact_expanded_names_ignore_prefix_and_reject_namespace_case_and_utf16_lookalikes() {
    let (mut b, d) = builder();
    let fs = element(
        &mut b,
        &d,
        QualifiedName::new(Namespace::html(), Some("p".into()), "fieldset"),
        OrderedAttributes::new([]).unwrap(),
    );
    let legend = element(
        &mut b,
        &d,
        QualifiedName::new(Namespace::html(), Some("q".into()), "legend"),
        OrderedAttributes::new([]).unwrap(),
    );
    children(&mut b, &fs, std::slice::from_ref(&legend));
    let foreign = element(
        &mut b,
        &d,
        QualifiedName::new(Namespace::svg(), None, "legend"),
        OrderedAttributes::new([]).unwrap(),
    );
    let upper = html(&mut b, &d, "LEGEND", &[]);
    let wrong = html(&mut b, &d, "div", &[]);
    let text = b
        .add_text(&d, DocumentString::from_utf16([0xd800]))
        .unwrap();
    let lone = element(
        &mut b,
        &d,
        QualifiedName::new(
            Namespace::html(),
            None,
            DocumentString::from_utf16([0xd800]),
        ),
        OrderedAttributes::new([]).unwrap(),
    );
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        &[(
            c.subject_id(&fs).unwrap(),
            HtmlFormAssociationState::EstablishedCurrent(None),
        )],
    );
    assert_eq!(
        value(&report(&c, &cx, &c.subject_id(&legend).unwrap(), 0), 0),
        &HtmlSemanticValue::Known(None)
    );
    assert_eq!(
        value(&report(&c, &cx, &c.subject_id(&foreign).unwrap(), 0), 0),
        &HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
    );
    for h in [&upper, &wrong, &text, &lone] {
        assert_eq!(
            value(&report(&c, &cx, &c.subject_id(h).unwrap(), 0), 0),
            &HtmlSemanticValue::Inapplicable
        );
    }
}

#[test]
fn foreign_fieldset_spelling_cannot_inherit_an_html_form_owner() {
    for ns in [Namespace::svg(), Namespace::none()] {
        let (mut b, d) = builder();
        let fs = element(
            &mut b,
            &d,
            QualifiedName::new(ns, None, "fieldset"),
            OrderedAttributes::new([]).unwrap(),
        );
        let legend = html(&mut b, &d, "legend", &[]);
        children(&mut b, &fs, std::slice::from_ref(&legend));
        let c = b.finish().unwrap();
        assert_eq!(
            value(
                &report(&c, &context(&c, &[]), &c.subject_id(&legend).unwrap(), 0),
                0
            ),
            &HtmlSemanticValue::Known(None)
        );
    }
}

#[test]
fn reset_without_form_attribute_uses_fieldset_ordinary_ancestor_or_established_root() {
    for attached in [true, false] {
        let (mut b, d) = builder();
        let f = html(&mut b, &d, "form", &[]);
        let fs = html(&mut b, &d, "fieldset", &[]);
        let legend = html(&mut b, &d, "legend", &[("form", "ignored")]);
        children(&mut b, &fs, std::slice::from_ref(&legend));
        if attached {
            children(&mut b, &f, std::slice::from_ref(&fs));
        } else {
            b.establish_root(&fs).unwrap();
        }
        let c = b.finish().unwrap();
        let fid = c.subject_id(&f).unwrap();
        let fsid = c.subject_id(&fs).unwrap();
        let cx = context(
            &c,
            &[(fsid.clone(), HtmlFormAssociationState::ResetAgainstCapture)],
        );
        let r = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
            .query(&[
                request(&c.subject_id(&legend).unwrap()),
                HtmlSemanticRequest::new(fsid, HtmlSemanticQuery::FormOwner),
            ])
            .unwrap();
        assert_eq!(
            value(&r, 0),
            &HtmlSemanticValue::Known(if attached { Some(fid) } else { None })
        );
        assert!(
            matches!(r.results()[1].fact(), HtmlSemanticFact::FormOwner(HtmlSemanticValue::Known(owner)) if owner.origin()==HtmlFormOwnerOrigin::ResetAgainstCapture)
        );
    }
}

fn reset_lookup(
    blocked: bool,
) -> (
    HtmlCapture,
    HtmlCaptureSubjectId,
    HtmlCaptureSubjectId,
    HtmlCaptureSubjectId,
) {
    let (mut b, d) = builder();
    let fs = html(&mut b, &d, "fieldset", &[("form", "f")]);
    let legend = html(&mut b, &d, "legend", &[("form", "oversized ignored")]);
    let first = html(
        &mut b,
        &d,
        if blocked { "div" } else { "form" },
        &[("id", "f")],
    );
    let later = html(&mut b, &d, "form", &[("id", "f")]);
    children(&mut b, &fs, std::slice::from_ref(&legend));
    children(&mut b, &legend, &[]);
    children(&mut b, &first, &[]);
    children(&mut b, &later, &[]);
    children(&mut b, &d, &[fs.clone(), first.clone(), later]);
    let c = b.finish().unwrap();
    let ids = (
        c.subject_id(&legend).unwrap(),
        c.subject_id(&fs).unwrap(),
        c.subject_id(&first).unwrap(),
    );
    (c, ids.0, ids.1, ids.2)
}

#[test]
fn reset_reuses_fieldset_form_raw_and_first_equal_id_owner_with_no_legend_override() {
    for blocked in [true, false] {
        let (c, legend, fs, first) = reset_lookup(blocked);
        let cx = context(&c, &[(fs, HtmlFormAssociationState::ResetAgainstCapture)]);
        assert_eq!(
            value(&report(&c, &cx, &legend, 1), 0),
            &HtmlSemanticValue::Known(if blocked { None } else { Some(first) })
        );
        let mut ss = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
        let f = ss.query(&[request(&legend)]).unwrap_err();
        assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
        assert_eq!(ss.query(&[]).unwrap_err(), f);
        assert_eq!(ss.usage().scratch_records(), 0);
    }
}

#[test]
fn reset_parent_missing_belongs_to_fieldset_not_legend() {
    let (mut b, d) = builder();
    let fs = html(&mut b, &d, "fieldset", &[]);
    let legend = html(&mut b, &d, "legend", &[]);
    b.establish_parent(&legend, &fs).unwrap();
    let c = b.finish().unwrap();
    let fsid = c.subject_id(&fs).unwrap();
    let cx = context(
        &c,
        &[(fsid.clone(), HtmlFormAssociationState::ResetAgainstCapture)],
    );
    assert_eq!(
        value(&report(&c, &cx, &c.subject_id(&legend).unwrap(), 0), 0),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(fsid))
    );
}

#[test]
fn unrelated_huge_values_text_and_unavailable_children_need_no_raw_or_scratch() {
    let (mut b, d) = builder();
    let huge = "x".repeat(10000);
    let fs = html(
        &mut b,
        &d,
        "fieldset",
        &[("form", &huge), ("disabled", &huge)],
    );
    let legend = html(&mut b, &d, "legend", &[("form", &huge), ("title", &huge)]);
    let text = b.add_text(&d, huge.into()).unwrap();
    children(&mut b, &legend, &[text]);
    b.establish_parent(&legend, &fs).unwrap();
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        &[(
            c.subject_id(&fs).unwrap(),
            HtmlFormAssociationState::EstablishedCurrent(None),
        )],
    );
    let r = session(
        &c,
        &cx,
        HtmlSemanticLimits::new(20000000, 1, 0, 0, 1000000, 0),
        0,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(&c.subject_id(&legend).unwrap())])
    .unwrap();
    assert_eq!(value(&r, 0), &HtmlSemanticValue::Known(None));
    assert_eq!(r.usage().peak_scratch_records(), 0);
    assert_eq!(r.usage().nested_output_records(), 0);
}

fn capped(resource: HtmlSemanticResource, cap: usize) -> HtmlSemanticLimits {
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
fn actual_reset_work_output_scratch_and_bytes_have_exact_one_short_sticky_limits() {
    let (c, legend, fs, form) = reset_lookup(false);
    let cx = context(&c, &[(fs, HtmlFormAssociationState::ResetAgainstCapture)]);
    let u = report(&c, &cx, &legend, 1).usage();
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
        let mut exact_session = session(
            &c,
            &cx,
            capped(resource, exact),
            1,
            HtmlSemanticCancellation::new(),
        );
        let r = exact_session.query(&[request(&legend)]).unwrap();
        assert_eq!(value(&r, 0), &HtmlSemanticValue::Known(Some(form.clone())));
        assert_eq!(exact_session.usage().scratch_records(), 0);
        let mut short = session(
            &c,
            &cx,
            capped(resource, exact - 1),
            1,
            HtmlSemanticCancellation::new(),
        );
        let f = short.query(&[request(&legend)]).unwrap_err();
        assert!(
            matches!(f.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r==resource),
            "{f:?}"
        );
        assert_eq!(short.query(&[]).unwrap_err(), f);
        assert_eq!(short.usage().scratch_records(), 0);
    }
}

#[test]
fn established_owner_uses_no_scratch_or_nested_records_and_physical_batch_storage_is_admitted() {
    let (c, legend, fs, form) = direct();
    let cx = context(
        &c,
        &[(
            fs,
            HtmlFormAssociationState::EstablishedCurrent(Some(form.clone())),
        )],
    );
    let l = HtmlSemanticLimits::new(20000000, 100, 0, 0, 1000000, 0);
    let mut ss = session(&c, &cx, l, 0, HtmlSemanticCancellation::new());
    let first = ss.query(&[request(&legend)]).unwrap();
    assert_eq!(
        first.usage().allocated_bytes(),
        std::mem::size_of::<HtmlSemanticResult>() * first.usage().output_records()
    );
    assert_eq!(first.usage().scratch_records(), 0);
    assert_eq!(first.usage().peak_scratch_records(), 0);
    assert_eq!(first.usage().nested_output_records(), 0);
    let second = ss.query(&[request(&legend)]).unwrap();
    assert_eq!(
        value(&second, 0),
        &HtmlSemanticValue::Known(Some(form.clone()))
    );
    assert_eq!(
        second.usage().allocated_bytes(),
        2 * first.usage().allocated_bytes()
    );
    assert_eq!(
        second.usage().output_records(),
        2 * first.usage().output_records()
    );
    let twice = session(&c, &cx, l, 0, HtmlSemanticCancellation::new())
        .query(&[request(&legend), request(&legend)])
        .unwrap();
    assert_eq!(twice.results().len(), 2);
    assert_eq!(
        value(&twice, 0),
        &HtmlSemanticValue::Known(Some(form.clone()))
    );
    assert_eq!(
        value(&twice, 1),
        &HtmlSemanticValue::Known(Some(form.clone()))
    );
    assert!(twice.usage().work() > first.usage().work());
    for (resource, exact) in [
        (
            HtmlSemanticResource::OutputRecords,
            twice.usage().output_records(),
        ),
        (
            HtmlSemanticResource::AllocatedBytes,
            twice.usage().allocated_bytes(),
        ),
    ] {
        let mut short = session(
            &c,
            &cx,
            capped(resource, exact - 1),
            0,
            HtmlSemanticCancellation::new(),
        );
        let f = short
            .query(&[request(&legend), request(&legend)])
            .unwrap_err();
        assert!(
            matches!(f.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r==resource)
        );
        assert_eq!(short.query(&[]).unwrap_err(), f);
    }
    assert_eq!(value(&first, 0), &HtmlSemanticValue::Known(Some(form)));
}

#[test]
fn missing_is_a_complete_fact_and_does_not_suppress_a_later_known_null() {
    let (mut b, d) = builder();
    let missing = html(&mut b, &d, "legend", &[]);
    let known = html(&mut b, &d, "legend", &[]);
    b.establish_root(&known).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    let m = c.subject_id(&missing).unwrap();
    let k = c.subject_id(&known).unwrap();
    let r = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
        .query(&[request(&m), request(&k)])
        .unwrap();
    assert_eq!(
        value(&r, 0),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(m))
    );
    assert_eq!(value(&r, 1), &HtmlSemanticValue::Known(None));
}

#[test]
fn late_raw_failure_discards_whole_batch_and_retains_original_qualification() {
    let (mut b, d) = builder();
    let first = html(&mut b, &d, "legend", &[]);
    b.establish_root(&first).unwrap();
    let fs = html(&mut b, &d, "fieldset", &[("form", "too long")]);
    let second = html(&mut b, &d, "legend", &[]);
    children(&mut b, &fs, std::slice::from_ref(&second));
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        &[(
            c.subject_id(&fs).unwrap(),
            HtmlFormAssociationState::ResetAgainstCapture,
        )],
    );
    let a = c.subject_id(&first).unwrap();
    let z = c.subject_id(&second).unwrap();
    let retained = report(&c, &cx, &a, 0);
    let mut ss = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let f = ss.query(&[request(&a), request(&z)]).unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(f.qualification().unwrap().capture(), c.identity());
    assert_eq!(f.qualification().unwrap().context(), cx.identity());
    assert_eq!(
        f.qualification().unwrap().host_revision(),
        c.host_facts_revision()
    );
    assert_eq!(ss.query(&[request(&a)]).unwrap_err(), f);
    assert_eq!(ss.usage().scratch_records(), 0);
    assert_eq!(value(&retained, 0), &HtmlSemanticValue::Known(None));
}

#[test]
fn cancellation_is_sticky_before_and_between_batches_with_exact_configuration() {
    let (c, legend, fs, form) = direct();
    let cx = context(
        &c,
        &[(
            fs,
            HtmlFormAssociationState::EstablishedCurrent(Some(form.clone())),
        )],
    );
    let cancel = HtmlSemanticCancellation::new();
    let token = cancel.clone();
    let mut ss = session(&c, &cx, limits(), 0, cancel);
    let retained = ss.query(&[request(&legend)]).unwrap();
    token.cancel();
    let f = ss.query(&[request(&legend)]).unwrap_err();
    assert_eq!(f.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(f.qualification().unwrap(), retained.qualification());
    assert_eq!(ss.query(&[]).unwrap_err(), f);
    assert_eq!(ss.usage().scratch_records(), 0);
    assert_eq!(value(&retained, 0), &HtmlSemanticValue::Known(Some(form)));
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let configured = capped(HtmlSemanticResource::Work, 999);
    let mut before = session(&c, &cx, configured, 77, cancel);
    let f = before.query(&[request(&legend)]).unwrap_err();
    assert_eq!(f.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(f.qualification().unwrap().limits(), configured);
    assert_eq!(f.qualification().unwrap().attribute_input_limit(), 77);
    assert_eq!(before.query(&[]).unwrap_err(), f);
}

#[test]
fn state_only_context_changes_recompute_owner_and_old_reports_survive() {
    let (c, legend, fs, form) = direct();
    let no = context(
        &c,
        &[(
            fs.clone(),
            HtmlFormAssociationState::EstablishedCurrent(None),
        )],
    );
    let yes = context(
        &c,
        &[(
            fs,
            HtmlFormAssociationState::EstablishedCurrent(Some(form.clone())),
        )],
    );
    let a = report(&c, &no, &legend, 0);
    let b = report(&c, &yes, &legend, 0);
    assert_eq!(value(&a, 0), &HtmlSemanticValue::Known(None));
    assert_eq!(value(&b, 0), &HtmlSemanticValue::Known(Some(form)));
    assert_eq!(a.capture(), b.capture());
    assert_ne!(a.context(), b.context());
    assert_eq!(a.host_revision(), c.host_facts_revision());
    assert_eq!(
        a.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
    assert_eq!(a.qualification().attribute_input_limit(), 0);
    assert_eq!(value(&a, 0), &HtmlSemanticValue::Known(None));
}

#[test]
fn different_capture_parent_and_host_revision_qualify_recomputation_and_foreign_keys_fail() {
    let (a, aid, afs, owner) = direct();
    let acx = context(
        &a,
        &[(
            afs,
            HtmlFormAssociationState::EstablishedCurrent(Some(owner.clone())),
        )],
    );
    let ar = report(&a, &acx, &aid, 0);
    let (mut b, d) = builder();
    let p = html(&mut b, &d, "div", &[]);
    let legend = html(&mut b, &d, "legend", &[]);
    children(&mut b, &p, std::slice::from_ref(&legend));
    let b = b.finish().unwrap();
    let bx = context(&b, &[]);
    let bid = b.subject_id(&legend).unwrap();
    let br = report(&b, &bx, &bid, 0);
    assert_eq!(value(&ar, 0), &HtmlSemanticValue::Known(Some(owner)));
    assert_eq!(value(&br, 0), &HtmlSemanticValue::Known(None));
    assert_ne!(ar.capture(), br.capture());
    assert_ne!(ar.host_revision(), br.host_revision());
    let mut ss = session(&a, &acx, limits(), 0, HtmlSemanticCancellation::new());
    let f = ss.query(&[request(&aid), request(&bid)]).unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Capture(_)));
    assert_eq!(ss.query(&[]).unwrap_err(), f);
    assert_eq!(ss.usage().scratch_records(), 0);
}

#[test]
fn namespaced_fieldset_form_is_ignored_but_selected_null_namespace_utf16_is_exact() {
    for namespaced in [true, false] {
        let (mut b, d) = builder();
        let name = DocumentString::from_utf16([0xd800, 0, 0x66]);
        let fs = element(
            &mut b,
            &d,
            QualifiedName::new(Namespace::html(), None, "fieldset"),
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(
                    if namespaced {
                        Namespace::svg()
                    } else {
                        Namespace::none()
                    },
                    None,
                    "form",
                ),
                name.clone(),
            )])
            .unwrap(),
        );
        let legend = html(&mut b, &d, "legend", &[]);
        let form = element(
            &mut b,
            &d,
            QualifiedName::new(Namespace::html(), None, "form"),
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), None, "id"),
                name,
            )])
            .unwrap(),
        );
        children(&mut b, &fs, std::slice::from_ref(&legend));
        children(&mut b, &legend, &[]);
        children(&mut b, &form, &[]);
        children(&mut b, &d, &[fs.clone(), form.clone()]);
        let c = b.finish().unwrap();
        let cx = context(
            &c,
            &[(
                c.subject_id(&fs).unwrap(),
                HtmlFormAssociationState::ResetAgainstCapture,
            )],
        );
        let id = c.subject_id(&legend).unwrap();
        let r = report(&c, &cx, &id, if namespaced { 0 } else { 3 });
        assert_eq!(
            value(&r, 0),
            &HtmlSemanticValue::Known(if namespaced {
                None
            } else {
                Some(c.subject_id(&form).unwrap())
            })
        );
        if !namespaced {
            let mut ss = session(&c, &cx, limits(), 2, HtmlSemanticCancellation::new());
            let f = ss.query(&[request(&id)]).unwrap_err();
            assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
            assert_eq!(ss.query(&[]).unwrap_err(), f);
            assert_eq!(ss.usage().scratch_records(), 0);
        }
    }
}

#[test]
fn source_clause_and_configuration_qualify_actual_readback() {
    let (c, legend, fs, _) = direct();
    let cx = context(
        &c,
        &[(fs, HtmlFormAssociationState::EstablishedCurrent(None))],
    );
    let one = report(&c, &cx, &legend, 0);
    let changed = HtmlSemanticLimits::new(limits().work() + 1, 100, 1000, 0, 1000000, 0);
    let two = session(&c, &cx, changed, 1, HtmlSemanticCancellation::new())
        .query(&[request(&legend)])
        .unwrap();
    assert_eq!(value(&one, 0), &HtmlSemanticValue::Known(None));
    assert_eq!(value(&two, 0), &HtmlSemanticValue::Known(None));
    assert_ne!(one.qualification(), two.qualification());
    assert_eq!(two.qualification().limits(), changed);
    assert_eq!(two.qualification().attribute_input_limit(), 1);
    assert_eq!(
        one.results()[0].request().query().source_clause(),
        "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-legend-form"
    );
    assert_eq!(
        one.source_revision(),
        "877d6146487f15b15bc8dd0e3c625063a37b456a"
    );
}
