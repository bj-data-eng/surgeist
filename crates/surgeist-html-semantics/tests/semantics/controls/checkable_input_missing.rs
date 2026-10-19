use crate::semantics::support::builder;
use crate::semantics::support::session;
use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlCapturedShadowProperties, HtmlControlFacts,
    HtmlControlFactsLimits, HtmlControlFlag, HtmlFormAssociation, HtmlFormAssociationState,
    HtmlInputControlState, HtmlSemanticCancellation, HtmlSemanticContext, HtmlSemanticContextInput,
    HtmlSemanticDependencies, HtmlSemanticFact, HtmlSemanticFailureKind, HtmlSemanticLimits,
    HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport, HtmlSemanticRequest,
    HtmlSemanticResource, HtmlSemanticUnsupported, HtmlSemanticValue,
};

fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    local: &str,
    attrs: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    expanded(
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
fn expanded(
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
fn children(
    b: &mut HtmlCaptureBuilder,
    p: &HtmlCaptureBuilderSubject,
    cs: &[HtmlCaptureBuilderSubject],
) {
    b.establish_children(p, cs).unwrap();
}
fn context(
    c: &HtmlCapture,
    states: &[(HtmlCaptureSubjectId, Option<bool>)],
    associations: Vec<HtmlFormAssociation>,
) -> HtmlSemanticContext {
    let facts = HtmlControlFacts::new(
        c,
        c.host_facts_revision().clone(),
        associations,
        states
            .iter()
            .map(|(id, value)| {
                HtmlInputControlState::new(id.clone(), *value, None, None, Some(true))
            })
            .collect(),
        vec![],
        vec![],
        HtmlControlFactsLimits::new(1000, 1000000),
    )
    .unwrap();
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_controls(facts),
        1000000,
    )
    .unwrap()
}
fn association(
    id: &HtmlCaptureSubjectId,
    owner: Option<&HtmlCaptureSubjectId>,
) -> HtmlFormAssociation {
    HtmlFormAssociation::new(
        id.clone(),
        HtmlFormAssociationState::EstablishedCurrent(owner.cloned()),
    )
}
fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20000000, 100, 1000, 1000000, 1000000, 100000)
}

fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::CheckableInputValueMissing)
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
fn fact(r: &HtmlSemanticReport) -> &HtmlSemanticValue<bool> {
    match r.results()[0].fact() {
        HtmlSemanticFact::CheckableInputValueMissing(v) => v,
        other => panic!("{other:?}"),
    }
}
fn fixture(attrs: &[(&str, &str)], complete: bool) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "input", attrs);
    if complete {
        children(&mut b, &e, &[]);
        b.establish_root(&e).unwrap();
    }
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    (c, id)
}
fn pair(
    a: &[(&str, &str)],
    z: &[(&str, &str)],
    gap: bool,
) -> (HtmlCapture, HtmlCaptureSubjectId, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let a = element(&mut b, &d, "input", a);
    let z = element(&mut b, &d, "input", z);
    children(&mut b, &a, &[]);
    children(&mut b, &z, &[]);
    if gap {
        let g = element(&mut b, &d, "div", &[]);
        children(&mut b, &d, &[a.clone(), g, z.clone()]);
    } else {
        children(&mut b, &d, &[a.clone(), z.clone()]);
    }
    let c = b.finish().unwrap();
    let a = c.subject_id(&a).unwrap();
    let z = c.subject_id(&z).unwrap();
    (c, a, z)
}

#[test]
fn checkbox_required_presence_uses_only_actual_checkedness() {
    for required in [false, true] {
        for checked in [Some(false), Some(true), None] {
            let attrs = if required {
                vec![
                    ("type", "checkbox"),
                    ("required", "malformed Boolean spelling"),
                    ("checked", ""),
                    ("disabled", "very long"),
                    ("value", "unread"),
                ]
            } else {
                vec![("type", "checkbox"), ("checked", "")]
            };
            let (c, id) = fixture(&attrs, false);
            let cx = context(&c, &[(id.clone(), checked)], vec![]);
            let r = report(&c, &cx, &id, 8);
            if required && checked.is_none() {
                assert_eq!(
                    fact(&r),
                    &HtmlSemanticValue::Missing(HtmlSemanticMissing::ControlState {
                        subject: id,
                        flag: HtmlControlFlag::Checkedness
                    })
                );
            } else {
                assert_eq!(
                    fact(&r),
                    &HtmlSemanticValue::Known(required && checked == Some(false))
                );
            }
            assert_eq!(r.usage().peak_scratch_records(), 0);
        }
    }
}
#[test]
fn unnamed_or_empty_name_radio_is_self_group_without_ancestry_or_owner() {
    for name in [None, Some("")] {
        for required in [false, true] {
            for checked in [Some(false), Some(true), None] {
                let mut attrs = vec![("type", "radio")];
                if let Some(name) = name {
                    attrs.push(("name", name));
                }
                if required {
                    attrs.push(("required", "oversized unread"));
                }
                let (c, id) = fixture(&attrs, false);
                let cx = context(&c, &[(id.clone(), checked)], vec![]);
                let r = report(&c, &cx, &id, 5);
                if required && checked.is_none() {
                    assert!(matches!(
                        fact(&r),
                        HtmlSemanticValue::Missing(HtmlSemanticMissing::ControlState { .. })
                    ));
                } else {
                    assert_eq!(
                        fact(&r),
                        &HtmlSemanticValue::Known(required && checked == Some(false))
                    );
                }
                assert_eq!(r.usage().peak_scratch_records(), 0);
            }
        }
    }
}
#[test]
fn disabled_required_radio_source_example_counts_for_every_group_member() {
    let (c, a, z) = pair(
        &[
            ("type", "radio"),
            ("name", "g"),
            ("required", ""),
            ("disabled", ""),
        ],
        &[("type", "radio"), ("name", "g")],
        false,
    );
    for selected in [false, true] {
        let cx = context(
            &c,
            &[(a.clone(), Some(false)), (z.clone(), Some(selected))],
            vec![association(&a, None), association(&z, None)],
        );
        for id in [&a, &z] {
            assert_eq!(
                fact(&report(&c, &cx, id, 5)),
                &HtmlSemanticValue::Known(!selected)
            );
        }
    }
}
#[test]
fn current_checked_self_skips_huge_names_form_and_unknown_tree() {
    let (c, id) = fixture(
        &[
            ("type", "radio"),
            ("required", "unread"),
            ("name", "huge unread name"),
            ("form", "huge unread form"),
        ],
        false,
    );
    let cx = context(&c, &[(id.clone(), Some(true))], vec![]);
    let r = report(&c, &cx, &id, 5);
    assert_eq!(fact(&r), &HtmlSemanticValue::Known(false));
    assert_eq!(r.usage().peak_scratch_records(), 0);
}
#[test]
fn checked_member_after_unknown_subtree_proves_false_and_releases_frames() {
    let (c, a, z) = pair(
        &[("type", "radio"), ("name", "g"), ("required", "")],
        &[("type", "radio"), ("name", "g")],
        true,
    );
    for own in [None, Some(false)] {
        let cx = context(
            &c,
            &[(a.clone(), own), (z.clone(), Some(true))],
            vec![association(&a, None), association(&z, None)],
        );
        let r = report(&c, &cx, &a, 5);
        assert_eq!(fact(&r), &HtmlSemanticValue::Known(false));
        assert_eq!(r.usage().scratch_records(), 0);
    }
}
#[test]
fn unavailable_children_cannot_be_treated_as_an_empty_group() {
    let (c, a, z) = pair(
        &[("type", "radio"), ("name", "g"), ("required", "")],
        &[("type", "radio"), ("name", "g")],
        true,
    );
    let cx = context(
        &c,
        &[(a.clone(), Some(false)), (z.clone(), Some(false))],
        vec![association(&a, None), association(&z, None)],
    );
    let r = report(&c, &cx, &a, 5);
    assert!(matches!(
        fact(&r),
        HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(_))
    ));
    assert_eq!(r.usage().scratch_records(), 0);
}
#[test]
fn complete_no_required_group_is_false_even_when_checkedness_is_unknown() {
    let (c, a, z) = pair(
        &[("type", "radio"), ("name", "g")],
        &[("type", "radio"), ("name", "g")],
        false,
    );
    let cx = context(
        &c,
        &[(a.clone(), None), (z.clone(), None)],
        vec![association(&a, None), association(&z, None)],
    );
    assert_eq!(
        fact(&report(&c, &cx, &a, 5)),
        &HtmlSemanticValue::Known(false)
    );
}
#[test]
fn possible_required_checked_member_keeps_its_inclusion_correlation() {
    let (c, a, z) = pair(
        &[("type", "radio"), ("name", "g")],
        &[("type", "radio"), ("name", "g"), ("required", "")],
        false,
    );
    for checked in [Some(true), Some(false), None] {
        let cx = context(
            &c,
            &[(a.clone(), Some(false)), (z.clone(), checked)],
            vec![association(&a, None)],
        );
        let r = report(&c, &cx, &a, 5);
        if checked == Some(true) {
            assert_eq!(fact(&r), &HtmlSemanticValue::Known(false));
        } else {
            assert!(
                matches!(fact(&r),HtmlSemanticValue::Missing(HtmlSemanticMissing::FormAssociation(id)) if id==&z)
            );
        }
    }
}
#[test]
fn neutral_unchecked_nonrequired_candidate_needs_no_form_pointer() {
    let (c, a, z) = pair(
        &[("type", "radio"), ("name", "g"), ("required", "")],
        &[("type", "radio"), ("name", "g"), ("form", "huge unused")],
        false,
    );
    let cx = context(
        &c,
        &[(a.clone(), Some(false)), (z.clone(), Some(false))],
        vec![],
    );
    assert_eq!(
        fact(&report(&c, &cx, &a, 5)),
        &HtmlSemanticValue::Known(true)
    );
}
#[test]
fn unknown_target_owner_is_not_null_but_can_be_irrelevant_to_no_required_proof() {
    for required in [false, true] {
        let mut attrs = vec![("type", "radio"), ("name", "g")];
        if required {
            attrs.push(("required", ""));
        }
        let (c, a, z) = pair(&[("type", "radio"), ("name", "g")], &attrs, false);
        let cx = context(
            &c,
            &[(a.clone(), Some(false)), (z.clone(), Some(false))],
            vec![association(&z, None)],
        );
        let r = report(&c, &cx, &a, 5);
        if required {
            assert!(
                matches!(fact(&r),HtmlSemanticValue::Missing(HtmlSemanticMissing::FormAssociation(id)) if id==&a)
            );
        } else {
            assert_eq!(fact(&r), &HtmlSemanticValue::Known(false));
        }
    }
}
#[test]
fn exact_name_comparison_has_no_trimming_or_case_folding() {
    for name in ["g", "G", " g", "g ", ""] {
        let (c, a, z) = pair(
            &[("type", "radio"), ("name", "g")],
            &[("type", "RADIO"), ("name", name), ("required", "")],
            false,
        );
        let cx = context(
            &c,
            &[(a.clone(), Some(false)), (z.clone(), Some(false))],
            vec![association(&a, None), association(&z, None)],
        );
        assert_eq!(
            fact(&report(&c, &cx, &a, 6)),
            &HtmlSemanticValue::Known(name == "g"),
            "{name:?}"
        );
    }
}
#[test]
fn different_actual_nonancestor_form_owner_excludes_checked_radio() {
    let (mut b, d) = builder();
    let f = element(&mut b, &d, "form", &[]);
    let g = element(&mut b, &d, "form", &[]);
    let a = element(
        &mut b,
        &d,
        "input",
        &[("type", "radio"), ("name", "g"), ("required", "")],
    );
    let z = element(&mut b, &d, "input", &[("type", "radio"), ("name", "g")]);
    for e in [&f, &g, &a, &z] {
        children(&mut b, e, &[]);
    }
    children(&mut b, &d, &[f.clone(), a.clone(), g.clone(), z.clone()]);
    let c = b.finish().unwrap();
    let a = c.subject_id(&a).unwrap();
    let z = c.subject_id(&z).unwrap();
    let f = c.subject_id(&f).unwrap();
    let g = c.subject_id(&g).unwrap();
    for same in [false, true] {
        let cx = context(
            &c,
            &[(a.clone(), Some(false)), (z.clone(), Some(true))],
            vec![
                association(&a, Some(&f)),
                association(&z, Some(if same { &f } else { &g })),
            ],
        );
        assert_eq!(
            fact(&report(&c, &cx, &a, 5)),
            &HtmlSemanticValue::Known(!same)
        );
    }
}
#[test]
fn separately_rooted_radios_do_not_join_even_with_equal_names_and_null_owners() {
    let (mut b, d) = builder();
    let a = element(
        &mut b,
        &d,
        "input",
        &[("type", "radio"), ("name", "g"), ("required", "")],
    );
    let z = element(&mut b, &d, "input", &[("type", "radio"), ("name", "g")]);
    for e in [&a, &z] {
        children(&mut b, e, &[]);
        b.establish_root(e).unwrap();
    }
    let c = b.finish().unwrap();
    let a = c.subject_id(&a).unwrap();
    let z = c.subject_id(&z).unwrap();
    let cx = context(
        &c,
        &[(a.clone(), Some(false)), (z.clone(), Some(true))],
        vec![association(&a, None), association(&z, None)],
    );
    assert_eq!(
        fact(&report(&c, &cx, &a, 5)),
        &HtmlSemanticValue::Known(true)
    );
}
#[test]
fn element_root_and_malformed_input_children_are_ordinary_group_members() {
    let (mut b, d) = builder();
    let a = element(
        &mut b,
        &d,
        "input",
        &[("type", "radio"), ("name", "g"), ("required", "")],
    );
    let z = element(&mut b, &d, "input", &[("type", "radio"), ("name", "g")]);
    children(&mut b, &z, &[]);
    children(&mut b, &a, std::slice::from_ref(&z));
    b.establish_root(&a).unwrap();
    let c = b.finish().unwrap();
    let a = c.subject_id(&a).unwrap();
    let z = c.subject_id(&z).unwrap();
    let cx = context(
        &c,
        &[(a.clone(), Some(true)), (z.clone(), Some(false))],
        vec![association(&a, None), association(&z, None)],
    );
    assert_eq!(
        fact(&report(&c, &cx, &z, 5)),
        &HtmlSemanticValue::Known(false)
    );
}
#[test]
fn noncheckable_roles_default_type_and_foreign_names_remain_distinct() {
    for (local, attrs) in [
        ("input", vec![]),
        ("input", vec![("type", "invalid")]),
        ("input", vec![("type", "text")]),
        ("Input", vec![("type", "radio")]),
        ("select", vec![]),
    ] {
        let (mut b, d) = builder();
        let e = element(&mut b, &d, local, &attrs);
        let c = b.finish().unwrap();
        let id = c.subject_id(&e).unwrap();
        assert_eq!(
            fact(&report(&c, &context(&c, &[], vec![]), &id, 10)),
            &HtmlSemanticValue::Inapplicable
        );
    }
    let (mut b, d) = builder();
    let e = expanded(
        &mut b,
        &d,
        Namespace::svg(),
        "input",
        OrderedAttributes::new([]).unwrap(),
    );
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let cx = context(&c, &[], vec![]);
    assert_eq!(
        fact(&report(&c, &cx, &id, 0)),
        &HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
    );
    assert_eq!(
        fact(&report(&c, &cx, &c.subject_id(&d).unwrap(), 0)),
        &HtmlSemanticValue::Inapplicable
    );
}

fn resource_limits(resource: HtmlSemanticResource, cap: usize) -> HtmlSemanticLimits {
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
        0,
    )
}
#[test]
fn exact_and_one_short_work_output_scratch_bytes_are_terminal_and_cleanup() {
    let (c, a, z) = pair(
        &[("type", "radio"), ("name", "g"), ("required", "")],
        &[("type", "radio"), ("name", "g")],
        false,
    );
    let cx = context(
        &c,
        &[(a.clone(), Some(false)), (z.clone(), Some(false))],
        vec![],
    );
    let usage = report(&c, &cx, &a, 5).usage();
    assert_eq!(usage.nested_output_records(), 0);
    for (resource, exact) in [
        (HtmlSemanticResource::Work, usage.work()),
        (HtmlSemanticResource::OutputRecords, usage.output_records()),
        (
            HtmlSemanticResource::ScratchRecords,
            usage.peak_scratch_records(),
        ),
        (
            HtmlSemanticResource::AllocatedBytes,
            usage.allocated_bytes(),
        ),
    ] {
        assert!(exact > 0);
        for cap in [exact, exact - 1] {
            let mut ss = session(
                &c,
                &cx,
                resource_limits(resource, cap),
                5,
                HtmlSemanticCancellation::new(),
            );
            let result = ss.query(&[request(&a)]);
            if cap == exact {
                assert_eq!(fact(&result.unwrap()), &HtmlSemanticValue::Known(true));
            } else {
                let f = result.unwrap_err();
                assert!(
                    matches!(f.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if r==&resource)
                );
                assert_eq!(f.qualification().unwrap().context(), cx.identity());
                assert_eq!(ss.query(&[]).unwrap_err(), f);
            }
            assert_eq!(ss.usage().scratch_records(), 0);
        }
    }
}
#[test]
fn consulted_whole_type_and_name_caps_preserve_original_failure() {
    for (attrs, cap) in [
        (vec![("type", "checkbox"), ("required", "")], 7),
        (vec![("type", "radio"), ("name", "long name")], 5),
    ] {
        let (c, id) = fixture(&attrs, true);
        let cx = context(&c, &[(id.clone(), Some(false))], vec![]);
        let mut ss = session(&c, &cx, limits(), cap, HtmlSemanticCancellation::new());
        let f = ss.query(&[request(&id)]).unwrap_err();
        assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
        assert_eq!(f.qualification().unwrap().attribute_input_limit(), cap);
        assert_eq!(ss.query(&[]).unwrap_err(), f);
        assert_eq!(ss.usage().scratch_records(), 0);
    }
}
#[test]
fn cancellation_and_late_batch_failure_return_no_partial_requested_results() {
    let (mut b, d) = builder();
    let a = element(&mut b, &d, "input", &[("type", "radio")]);
    let z = element(&mut b, &d, "input", &[("type", "checkbox")]);
    let c = b.finish().unwrap();
    let a = c.subject_id(&a).unwrap();
    let z = c.subject_id(&z).unwrap();
    let cx = context(&c, &[(a.clone(), Some(true))], vec![]);
    let mut ss = session(&c, &cx, limits(), 5, HtmlSemanticCancellation::new());
    let f = ss.query(&[request(&a), request(&z)]).unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(ss.query(&[]).unwrap_err(), f);
    assert_eq!(ss.usage().scratch_records(), 0);
    let cancel = HtmlSemanticCancellation::new();
    let mut ss = session(&c, &cx, limits(), 20, cancel.clone());
    cancel.cancel();
    let f = ss.query(&[request(&a)]).unwrap_err();
    assert_eq!(f.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(f.qualification().unwrap().capture(), c.identity());
    assert_eq!(ss.query(&[]).unwrap_err(), f);
}
#[test]
fn duplicate_queries_recompute_and_cumulate_cost_without_retaining_group_vectors() {
    let (c, a, z) = pair(
        &[("type", "radio"), ("name", "g"), ("required", "")],
        &[("type", "radio"), ("name", "g")],
        false,
    );
    let cx = context(&c, &[(a.clone(), Some(false)), (z, Some(false))], vec![]);
    let one = report(&c, &cx, &a, 5);
    let mut ss = session(&c, &cx, limits(), 5, HtmlSemanticCancellation::new());
    let two = ss.query(&[request(&a), request(&a)]).unwrap();
    assert_eq!(two.results().len(), 2);
    assert!(two.usage().work() > one.usage().work());
    assert!(two.usage().allocated_bytes() > one.usage().allocated_bytes());
    assert_eq!(two.usage().nested_output_records(), 0);
    assert_eq!(two.usage().scratch_records(), 0);
    assert_eq!(
        two.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
    let mut ss = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::Work, two.usage().work() - 1),
        5,
        HtmlSemanticCancellation::new(),
    );
    let f = ss.query(&[request(&a), request(&a)]).unwrap_err();
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            ..
        }
    ));
    assert_eq!(ss.usage().scratch_records(), 0);
}
#[test]
fn changed_current_state_has_distinct_context_qualification_and_recomputed_result() {
    let (c, id) = fixture(&[("type", "checkbox"), ("required", "")], false);
    let a = context(&c, &[(id.clone(), Some(false))], vec![]);
    let z = context(&c, &[(id.clone(), Some(true))], vec![]);
    let x = report(&c, &a, &id, 8);
    let y = report(&c, &z, &id, 8);
    assert_eq!(fact(&x), &HtmlSemanticValue::Known(true));
    assert_eq!(fact(&y), &HtmlSemanticValue::Known(false));
    assert_ne!(x.qualification().context(), y.qualification().context());
    assert_eq!(x.qualification().capture(), c.identity());
    let (other, foreign) = fixture(&[("type", "radio")], true);
    let mut ss = session(&c, &a, limits(), 8, HtmlSemanticCancellation::new());
    let f = ss.query(&[request(&foreign)]).unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Capture(_)));
    assert_ne!(other.identity(), c.identity());
    assert_eq!(f.qualification().unwrap().context(), a.identity());
}

#[test]
fn names_preserve_nuls_and_unpaired_surrogates_and_ignore_namespaced_lookalikes() {
    for equal in [false, true] {
        let (mut b, d) = builder();
        let attrs = |name: DocumentString| {
            OrderedAttributes::new([
                Attribute::new(QualifiedName::new(Namespace::none(), None, "type"), "radio"),
                Attribute::new(QualifiedName::new(Namespace::none(), None, "name"), name),
            ])
            .unwrap()
        };
        let a = expanded(
            &mut b,
            &d,
            Namespace::html(),
            "input",
            attrs(DocumentString::from_utf16(vec![0xd800, 0, 0x61])),
        );
        let z = expanded(
            &mut b,
            &d,
            Namespace::html(),
            "input",
            OrderedAttributes::new([
                Attribute::new(QualifiedName::new(Namespace::none(), None, "type"), "radio"),
                Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "name"),
                    DocumentString::from_utf16(vec![0xd800, 0, if equal { 0x61 } else { 0x41 }]),
                ),
                Attribute::new(QualifiedName::new(Namespace::none(), None, "required"), ""),
            ])
            .unwrap(),
        );
        children(&mut b, &a, &[]);
        children(&mut b, &z, &[]);
        children(&mut b, &d, &[a.clone(), z.clone()]);
        let c = b.finish().unwrap();
        let a = c.subject_id(&a).unwrap();
        let z = c.subject_id(&z).unwrap();
        let cx = context(
            &c,
            &[(a.clone(), Some(false)), (z.clone(), Some(false))],
            vec![association(&a, None), association(&z, None)],
        );
        assert_eq!(
            fact(&report(&c, &cx, &a, 5)),
            &HtmlSemanticValue::Known(equal)
        );
    }
    let (mut b, d) = builder();
    let a = expanded(
        &mut b,
        &d,
        Namespace::html(),
        "input",
        OrderedAttributes::new([
            Attribute::new(
                QualifiedName::new(Namespace::none(), None, "type"),
                "checkbox",
            ),
            Attribute::new(
                QualifiedName::new(Namespace::svg(), None, "required"),
                "huge unread",
            ),
        ])
        .unwrap(),
    );
    let c = b.finish().unwrap();
    let a = c.subject_id(&a).unwrap();
    assert_eq!(
        fact(&report(&c, &context(&c, &[], vec![]), &a, 8)),
        &HtmlSemanticValue::Known(false)
    );
}

#[test]
fn reset_associations_use_existing_source_owner_and_preserve_form_raw_admission() {
    let (mut b, d) = builder();
    let f = element(&mut b, &d, "form", &[("id", "owner")]);
    let a = element(
        &mut b,
        &d,
        "input",
        &[("type", "radio"), ("name", "g"), ("required", "")],
    );
    let z = element(
        &mut b,
        &d,
        "input",
        &[("type", "radio"), ("name", "g"), ("form", "owner")],
    );
    children(&mut b, &a, &[]);
    children(&mut b, &z, &[]);
    children(&mut b, &f, std::slice::from_ref(&a));
    children(&mut b, &d, &[f.clone(), z.clone()]);
    let c = b.finish().unwrap();
    let a = c.subject_id(&a).unwrap();
    let z = c.subject_id(&z).unwrap();
    let cx = context(
        &c,
        &[(a.clone(), Some(false)), (z.clone(), Some(true))],
        vec![
            HtmlFormAssociation::new(a.clone(), HtmlFormAssociationState::ResetAgainstCapture),
            HtmlFormAssociation::new(z.clone(), HtmlFormAssociationState::ResetAgainstCapture),
        ],
    );
    assert_eq!(
        fact(&report(&c, &cx, &a, 5)),
        &HtmlSemanticValue::Known(false)
    );
    let (c, a, z) = pair(
        &[("type", "radio"), ("name", "g"), ("required", "")],
        &[("type", "radio"), ("name", "g"), ("form", "owner-long")],
        false,
    );
    let cx = context(
        &c,
        &[(a.clone(), Some(false)), (z.clone(), Some(true))],
        vec![
            association(&a, None),
            HtmlFormAssociation::new(z, HtmlFormAssociationState::ResetAgainstCapture),
        ],
    );
    let mut ss = session(&c, &cx, limits(), 5, HtmlSemanticCancellation::new());
    let f = ss.query(&[request(&a)]).unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(ss.usage().scratch_records(), 0);
    assert_eq!(ss.query(&[]).unwrap_err(), f);
}

#[test]
fn checked_nonradio_or_unequal_name_does_not_resolve_influencing_unknown_checkedness() {
    for attrs in [
        vec![("type", "checkbox"), ("name", "g")],
        vec![("type", "radio"), ("name", "other")],
    ] {
        let (c, a, z) = pair(
            &[("type", "radio"), ("name", "g"), ("required", "")],
            &attrs,
            false,
        );
        let cx = context(&c, &[(a.clone(), None), (z, Some(true))], vec![]);
        assert_eq!(
            fact(&report(&c, &cx, &a, 8)),
            &HtmlSemanticValue::Missing(HtmlSemanticMissing::ControlState {
                subject: a,
                flag: HtmlControlFlag::Checkedness
            })
        );
    }
}

#[test]
fn known_required_evidence_supersedes_missing_required_only_pointer_evidence() {
    let (mut b, d) = builder();
    let a = element(&mut b, &d, "input", &[("type", "radio"), ("name", "g")]);
    let z = element(
        &mut b,
        &d,
        "input",
        &[("type", "radio"), ("name", "g"), ("required", "")],
    );
    let r = element(
        &mut b,
        &d,
        "input",
        &[("type", "radio"), ("name", "g"), ("required", "")],
    );
    for e in [&a, &z, &r] {
        children(&mut b, e, &[]);
    }
    children(&mut b, &d, &[a.clone(), z.clone(), r.clone()]);
    let c = b.finish().unwrap();
    let a = c.subject_id(&a).unwrap();
    let z = c.subject_id(&z).unwrap();
    let r = c.subject_id(&r).unwrap();
    let cx = context(
        &c,
        &[
            (a.clone(), None),
            (z, Some(false)),
            (r.clone(), Some(false)),
        ],
        vec![association(&a, None), association(&r, None)],
    );
    assert_eq!(
        fact(&report(&c, &cx, &a, 5)),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::ControlState {
            subject: a,
            flag: HtmlControlFlag::Checkedness
        })
    );
}

#[test]
fn possible_membership_on_the_query_pointer_keeps_checked_and_required_correlated() {
    let (c, a, z) = pair(
        &[("type", "radio"), ("name", "g")],
        &[("type", "radio"), ("name", "g"), ("required", "")],
        false,
    );
    let cx = context(
        &c,
        &[(a.clone(), Some(false)), (z.clone(), Some(true))],
        vec![association(&z, None)],
    );
    assert_eq!(
        fact(&report(&c, &cx, &a, 5)),
        &HtmlSemanticValue::Known(false)
    );
}

#[test]
fn shadow_and_template_associations_never_replace_ordinary_group_edges() {
    for template in [false, true] {
        let (mut b, d) = builder();
        let host = element(&mut b, &d, if template { "template" } else { "div" }, &[]);
        let a = element(
            &mut b,
            &d,
            "input",
            &[("type", "radio"), ("name", "g"), ("required", "")],
        );
        let z = element(&mut b, &d, "input", &[("type", "radio"), ("name", "g")]);
        let separate = if template {
            let fragment = b.add_fragment(&d).unwrap();
            b.establish_template_contents(&host, &fragment).unwrap();
            fragment
        } else {
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
        };
        children(&mut b, &a, &[]);
        children(&mut b, &z, &[]);
        children(&mut b, &separate, std::slice::from_ref(&z));
        children(&mut b, &host, &[]);
        children(&mut b, &d, &[host, a.clone()]);
        let c = b.finish().unwrap();
        let a = c.subject_id(&a).unwrap();
        let z = c.subject_id(&z).unwrap();
        let cx = context(
            &c,
            &[(a.clone(), Some(false)), (z.clone(), Some(true))],
            vec![association(&a, None), association(&z, None)],
        );
        assert_eq!(
            fact(&report(&c, &cx, &a, 5)),
            &HtmlSemanticValue::Known(true)
        );
    }
}

#[test]
fn unknown_parent_is_influencing_for_named_group_but_not_a_checked_self() {
    let (c, id) = fixture(&[("type", "radio"), ("name", "g"), ("required", "")], false);
    for checked in [false, true] {
        let cx = context(&c, &[(id.clone(), Some(checked))], vec![]);
        let r = report(&c, &cx, &id, 5);
        if checked {
            assert_eq!(fact(&r), &HtmlSemanticValue::Known(false));
        } else {
            assert_eq!(
                fact(&r),
                &HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(id.clone()))
            );
        }
    }
}

#[test]
fn late_confirmed_checked_proof_is_still_subject_to_real_work_and_scratch_limits() {
    let (c, a, z) = pair(
        &[("type", "radio"), ("name", "g"), ("required", "")],
        &[("type", "radio"), ("name", "g")],
        true,
    );
    let cx = context(
        &c,
        &[(a.clone(), None), (z.clone(), Some(true))],
        vec![association(&a, None), association(&z, None)],
    );
    let baseline = report(&c, &cx, &a, 5);
    for (resource, cap) in [
        (HtmlSemanticResource::Work, baseline.usage().work() - 1),
        (
            HtmlSemanticResource::ScratchRecords,
            baseline.usage().peak_scratch_records() - 1,
        ),
    ] {
        let mut ss = session(
            &c,
            &cx,
            resource_limits(resource, cap),
            5,
            HtmlSemanticCancellation::new(),
        );
        let f = ss.query(&[request(&a)]).unwrap_err();
        assert!(
            matches!(f.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if r==&resource)
        );
        assert_eq!(ss.usage().scratch_records(), 0);
        assert_eq!(ss.query(&[]).unwrap_err(), f);
    }
}

#[test]
fn session_host_and_configuration_keys_are_retained_on_both_success_and_stop() {
    let (c, id) = fixture(&[("type", "checkbox"), ("required", "")], false);
    let cx = context(&c, &[(id.clone(), Some(false))], vec![]);
    let r = report(&c, &cx, &id, 8);
    assert_eq!(r.qualification().host_revision(), c.host_facts_revision());
    assert_eq!(r.qualification().attribute_input_limit(), 8);
    let l = resource_limits(HtmlSemanticResource::Work, 0);
    let mut ss = session(&c, &cx, l, 9, HtmlSemanticCancellation::new());
    let f = ss.query(&[request(&id)]).unwrap_err();
    let key = f.qualification().unwrap();
    assert_eq!(key.host_revision(), c.host_facts_revision());
    assert_eq!(key.limits(), l);
    assert_eq!(key.attribute_input_limit(), 9);
    assert_eq!(key.context(), cx.identity());
    assert_eq!(ss.query(&[]).unwrap_err(), f);
}

#[test]
fn later_confirmed_required_member_leaves_only_real_checkedness_uncertainty() {
    let (mut b, d) = builder();
    let a = element(&mut b, &d, "input", &[("type", "radio"), ("name", "g")]);
    let possible = element(
        &mut b,
        &d,
        "input",
        &[("type", "radio"), ("name", "g"), ("required", "")],
    );
    let confirmed = element(
        &mut b,
        &d,
        "input",
        &[("type", "radio"), ("name", "g"), ("required", "")],
    );
    for e in [&a, &possible, &confirmed] {
        children(&mut b, e, &[]);
    }
    children(
        &mut b,
        &d,
        &[a.clone(), possible.clone(), confirmed.clone()],
    );
    let c = b.finish().unwrap();
    let a = c.subject_id(&a).unwrap();
    let possible = c.subject_id(&possible).unwrap();
    let confirmed = c.subject_id(&confirmed).unwrap();
    let cx = context(
        &c,
        &[
            (a.clone(), Some(false)),
            (possible, Some(false)),
            (confirmed.clone(), None),
        ],
        vec![association(&a, None), association(&confirmed, None)],
    );
    assert_eq!(
        fact(&report(&c, &cx, &a, 5)),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::ControlState {
            subject: confirmed,
            flag: HtmlControlFlag::Checkedness
        })
    );
}

#[test]
fn checked_group_witness_does_not_inspect_its_children_or_later_oversized_type() {
    let (mut b, d) = builder();
    let a = element(
        &mut b,
        &d,
        "input",
        &[("type", "radio"), ("name", "g"), ("required", "")],
    );
    let checked = element(
        &mut b,
        &d,
        "input",
        &[
            ("type", "radio"),
            ("name", "g"),
            ("value", "huge unread"),
            ("disabled", "huge unread"),
        ],
    );
    let later = element(&mut b, &d, "input", &[("type", "oversized unused type")]);
    children(&mut b, &a, &[]);
    children(&mut b, &d, &[a.clone(), checked.clone(), later]);
    let c = b.finish().unwrap();
    let a = c.subject_id(&a).unwrap();
    let checked = c.subject_id(&checked).unwrap();
    let cx = context(
        &c,
        &[(a.clone(), Some(false)), (checked.clone(), Some(true))],
        vec![association(&a, None), association(&checked, None)],
    );
    assert_eq!(
        fact(&report(&c, &cx, &a, 5)),
        &HtmlSemanticValue::Known(false)
    );
}

#[test]
fn absent_state_supplement_is_missing_while_missing_or_namespaced_type_defaults_to_text() {
    let (c, id) = fixture(&[("type", "checkbox"), ("required", "")], false);
    let cx = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]),
        0,
    )
    .unwrap();
    assert_eq!(
        fact(&report(&c, &cx, &id, 8)),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::ControlState {
            subject: id,
            flag: HtmlControlFlag::Checkedness
        })
    );
    let (c, id) = fixture(&[("required", "huge unread")], false);
    assert_eq!(
        fact(&report(&c, &context(&c, &[], vec![]), &id, 0)),
        &HtmlSemanticValue::Inapplicable
    );
    let (mut b, d) = builder();
    let e = expanded(
        &mut b,
        &d,
        Namespace::html(),
        "input",
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::svg(), None, "type"),
            "huge unread",
        )])
        .unwrap(),
    );
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    assert_eq!(
        fact(&report(&c, &context(&c, &[], vec![]), &id, 0)),
        &HtmlSemanticValue::Inapplicable
    );
}

#[test]
fn full_original_name_admission_succeeds_at_exact_units_without_numeric_or_nested_output() {
    let (c, id) = fixture(&[("type", "radio"), ("name", "long name")], true);
    let cx = context(&c, &[(id.clone(), Some(false))], vec![]);
    let mut ss = session(
        &c,
        &cx,
        HtmlSemanticLimits::new(20000000, 100, 1000, 0, 1000000, 0),
        9,
        HtmlSemanticCancellation::new(),
    );
    let r = ss.query(&[request(&id)]).unwrap();
    assert_eq!(fact(&r), &HtmlSemanticValue::Known(false));
    assert_eq!(r.usage().nested_output_records(), 0);
    let mut ss = session(&c, &cx, limits(), 8, HtmlSemanticCancellation::new());
    let f = ss.query(&[request(&id)]).unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
}
