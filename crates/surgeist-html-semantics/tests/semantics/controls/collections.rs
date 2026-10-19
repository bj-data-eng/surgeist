use crate::semantics::support::{
    COLLECTION_RESOURCES, cap_resources, compact_builder_without_registries as builder,
    moderate_semantic_limits_with_scratch as limits,
};
use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureError,
    HtmlCaptureRegistryInput, HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlCapturedNodeKind,
    HtmlCapturedShadowProperties, HtmlControlFacts, HtmlControlFactsLimits,
    HtmlCustomControlClassification, HtmlFormAssociation, HtmlFormAssociationState,
    HtmlListedControlCollection, HtmlSemanticCancellation, HtmlSemanticContext,
    HtmlSemanticContextInput, HtmlSemanticDependencies, HtmlSemanticFact, HtmlSemanticFailureKind,
    HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport,
    HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticSession, HtmlSemanticUnsupported,
    HtmlSemanticValue,
};

fn attrs(values: &[(&str, &str)]) -> OrderedAttributes {
    OrderedAttributes::new(
        values
            .iter()
            .map(|(n, v)| Attribute::new(QualifiedName::new(Namespace::none(), None, *n), *v)),
    )
    .unwrap()
}
fn expanded(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    ns: Namespace,
    local: &str,
    a: OrderedAttributes,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(ns, None, local),
        a,
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    local: &str,
    a: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    expanded(b, d, Namespace::html(), local, attrs(a))
}
struct Fixture {
    capture: HtmlCapture,
    ids: Vec<HtmlCaptureSubjectId>,
}
fn finish(b: HtmlCaptureBuilder, handles: &[HtmlCaptureBuilderSubject]) -> Fixture {
    let capture = b.finish().unwrap();
    let ids = handles
        .iter()
        .map(|h| capture.subject_id(h).unwrap())
        .collect();
    Fixture { capture, ids }
}
fn flat(specs: &[(&str, &[(&str, &str)])], order: &[usize]) -> Fixture {
    let (mut b, d) = builder();
    let h: Vec<_> = specs
        .iter()
        .map(|(n, a)| element(&mut b, &d, n, a))
        .collect();
    for v in &h {
        b.establish_children(v, &[]).unwrap();
    }
    b.establish_children(&d, &order.iter().map(|i| h[*i].clone()).collect::<Vec<_>>())
        .unwrap();
    finish(b, &h)
}
fn context(
    f: &Fixture,
    states: &[(usize, HtmlFormAssociationState)],
    classifications: &[(usize, bool)],
) -> HtmlSemanticContext {
    let facts = HtmlControlFacts::new(
        &f.capture,
        f.capture.host_facts_revision().clone(),
        states
            .iter()
            .map(|(i, s)| HtmlFormAssociation::new(f.ids[*i].clone(), s.clone()))
            .collect(),
        vec![],
        vec![],
        vec![],
        HtmlControlFactsLimits::new(1000, 1000000),
    )
    .unwrap();
    HtmlSemanticContext::new(
        &f.capture,
        f.capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(
            classifications
                .iter()
                .map(|(i, v)| HtmlCustomControlClassification::new(f.ids[*i].clone(), *v))
                .collect(),
        )
        .with_controls(facts),
        1000,
    )
    .unwrap()
}
fn owned(f: &Fixture, target: usize, members: &[usize]) -> HtmlSemanticContext {
    context(
        f,
        &members
            .iter()
            .map(|i| {
                (
                    *i,
                    HtmlFormAssociationState::EstablishedCurrent(Some(f.ids[target].clone())),
                )
            })
            .collect::<Vec<_>>(),
        &[],
    )
}
fn session<'a>(
    f: &'a Fixture,
    c: &'a HtmlSemanticContext,
    l: HtmlSemanticLimits,
    raw: usize,
    cancel: HtmlSemanticCancellation,
) -> HtmlSemanticSession<'a> {
    HtmlSemanticSession::new(&f.capture, c, HtmlValueInterpreter::new(raw), l, cancel).unwrap()
}
fn report(
    f: &Fixture,
    c: &HtmlSemanticContext,
    target: usize,
    q: HtmlSemanticQuery,
) -> HtmlSemanticReport {
    session(f, c, limits(), 10000, HtmlSemanticCancellation::new())
        .query(&[HtmlSemanticRequest::new(f.ids[target].clone(), q)])
        .unwrap()
}
fn collection(r: &HtmlSemanticReport) -> &HtmlSemanticValue<HtmlListedControlCollection> {
    match r.results()[0].fact() {
        HtmlSemanticFact::FormControls(v) | HtmlSemanticFact::FieldsetControls(v) => v,
        other => panic!("unexpected {other:?}"),
    }
}
fn controls(r: &HtmlSemanticReport) -> &[HtmlCaptureSubjectId] {
    match collection(r) {
        HtmlSemanticValue::Known(v) => {
            assert_eq!(v.len(), v.controls().len());
            assert_eq!(v.is_empty(), v.controls().is_empty());
            v.controls()
        }
        other => panic!("expected complete membership {other:?}"),
    }
}
fn length(r: &HtmlSemanticReport) -> &HtmlSemanticValue<usize> {
    match r.results()[0].fact() {
        HtmlSemanticFact::FormLength(v) => v,
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn externally_associated_controls_before_and_after_form_are_in_tree_order() {
    let f = flat(
        &[("form", &[]), ("input", &[]), ("button", &[])],
        &[2, 0, 1],
    );
    let c = owned(&f, 0, &[1, 2]);
    let r = report(&f, &c, 0, HtmlSemanticQuery::FormControls);
    assert_eq!(controls(&r), &[f.ids[2].clone(), f.ids[1].clone()]);
    assert_eq!(
        length(&report(&f, &c, 0, HtmlSemanticQuery::FormLength)),
        &HtmlSemanticValue::Known(2)
    );
    assert_eq!(
        r.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
}

#[test]
fn all_seven_native_listed_roles_participate_but_other_form_related_roles_do_not() {
    let f = flat(
        &[
            ("form", &[]),
            ("button", &[]),
            ("fieldset", &[]),
            ("input", &[("type", "hidden")]),
            ("object", &[]),
            ("output", &[]),
            ("select", &[]),
            ("textarea", &[]),
            ("img", &[]),
            ("option", &[]),
            ("optgroup", &[]),
            ("meter", &[]),
            ("progress", &[]),
            ("label", &[]),
        ],
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
    );
    let c = owned(&f, 0, &[1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(
        controls(&report(&f, &c, 0, HtmlSemanticQuery::FormControls)),
        &f.ids[1..8]
    );
    assert_eq!(
        length(&report(&f, &c, 0, HtmlSemanticQuery::FormLength)),
        &HtmlSemanticValue::Known(7)
    );
}

#[test]
fn fieldset_excludes_itself_but_includes_nested_fieldset_and_image_input() {
    let (mut b, d) = builder();
    let outer = element(&mut b, &d, "fieldset", &[("disabled", "false")]);
    let legend = element(&mut b, &d, "legend", &[]);
    let input = element(
        &mut b,
        &d,
        "input",
        &[("type", "image"), ("form", "oversized-unread")],
    );
    let inner = element(&mut b, &d, "fieldset", &[]);
    let button = element(&mut b, &d, "button", &[]);
    let img = element(&mut b, &d, "img", &[]);
    for h in [&input, &button, &img] {
        b.establish_children(h, &[]).unwrap();
    }
    b.establish_children(&legend, std::slice::from_ref(&input))
        .unwrap();
    b.establish_children(&inner, std::slice::from_ref(&button))
        .unwrap();
    b.establish_children(&outer, &[legend, inner.clone(), img])
        .unwrap();
    b.establish_children(&d, std::slice::from_ref(&outer))
        .unwrap();
    let f = finish(b, &[outer, input, inner, button]);
    let c = context(&f, &[], &[]);
    let r = session(&f, &c, limits(), 0, HtmlSemanticCancellation::new())
        .query(&[HtmlSemanticRequest::new(
            f.ids[0].clone(),
            HtmlSemanticQuery::FieldsetControls,
        )])
        .unwrap();
    assert_eq!(controls(&r), &f.ids[1..]);
}

#[test]
fn empty_fieldset_and_empty_form_are_known_empty_and_count_is_zero() {
    for (local, q) in [
        ("form", HtmlSemanticQuery::FormControls),
        ("fieldset", HtmlSemanticQuery::FieldsetControls),
    ] {
        let f = flat(&[(local, &[])], &[0]);
        let c = context(&f, &[], &[]);
        assert!(controls(&report(&f, &c, 0, q)).is_empty());
        if local == "form" {
            assert_eq!(
                length(&report(&f, &c, 0, HtmlSemanticQuery::FormLength)),
                &HtmlSemanticValue::Known(0)
            );
        }
    }
}

#[test]
fn fieldset_ignores_other_owners_and_does_not_list_external_associations() {
    let (mut b, d) = builder();
    let fieldset = element(&mut b, &d, "fieldset", &[]);
    let form = element(&mut b, &d, "form", &[]);
    let child = element(&mut b, &d, "input", &[]);
    let external = element(&mut b, &d, "button", &[]);
    for h in [&child, &external, &form] {
        b.establish_children(h, &[]).unwrap();
    }
    b.establish_children(&fieldset, std::slice::from_ref(&child))
        .unwrap();
    b.establish_children(&d, &[external.clone(), form.clone(), fieldset.clone()])
        .unwrap();
    let f = finish(b, &[fieldset, form, child, external]);
    let c = context(
        &f,
        &[
            (0, HtmlFormAssociationState::EstablishedCurrent(None)),
            (
                2,
                HtmlFormAssociationState::EstablishedCurrent(Some(f.ids[1].clone())),
            ),
            (
                3,
                HtmlFormAssociationState::EstablishedCurrent(Some(f.ids[1].clone())),
            ),
        ],
        &[],
    );
    assert_eq!(
        controls(&report(&f, &c, 0, HtmlSemanticQuery::FieldsetControls)),
        std::slice::from_ref(&f.ids[2])
    );
    assert_eq!(
        controls(&report(&f, &c, 1, HtmlSemanticQuery::FormControls)),
        &[f.ids[3].clone(), f.ids[2].clone()]
    );
}

#[test]
fn established_nonancestor_owner_and_reset_owner_have_distinct_membership() {
    let (mut b, d) = builder();
    let target = element(&mut b, &d, "form", &[]);
    let ancestor = element(&mut b, &d, "form", &[]);
    let button = element(&mut b, &d, "button", &[]);
    b.establish_children(&target, &[]).unwrap();
    b.establish_children(&button, &[]).unwrap();
    b.establish_children(&ancestor, std::slice::from_ref(&button))
        .unwrap();
    b.establish_children(&d, &[target.clone(), ancestor.clone()])
        .unwrap();
    let f = finish(b, &[target, button, ancestor]);
    let history = owned(&f, 0, &[1]);
    let reset = context(
        &f,
        &[(1, HtmlFormAssociationState::ResetAgainstCapture)],
        &[],
    );
    let r = report(&f, &history, 0, HtmlSemanticQuery::FormControls);
    let other = report(&f, &reset, 0, HtmlSemanticQuery::FormControls);
    assert_eq!(controls(&r), std::slice::from_ref(&f.ids[1]));
    assert!(controls(&other).is_empty());
    assert_ne!(r.qualification().context(), other.qualification().context());
    assert_eq!(r.capture(), other.capture());
}

#[test]
fn reset_external_owner_uses_first_equal_id_not_first_matching_form() {
    for obstruct in [false, true] {
        let f = flat(
            &[
                ("form", &[("id", "target")]),
                ("div", &[("id", "target")]),
                ("input", &[("form", "target")]),
            ],
            if obstruct { &[1, 2, 0] } else { &[2, 0, 1] },
        );
        let c = context(
            &f,
            &[(2, HtmlFormAssociationState::ResetAgainstCapture)],
            &[],
        );
        let r = report(&f, &c, 0, HtmlSemanticQuery::FormControls);
        if obstruct {
            assert!(controls(&r).is_empty());
        } else {
            assert_eq!(controls(&r), std::slice::from_ref(&f.ids[2]));
        }
    }
}

#[test]
fn null_is_nonmembership_but_unavailable_owner_is_missing_not_a_prefix() {
    let f = flat(
        &[("form", &[]), ("button", &[]), ("textarea", &[])],
        &[0, 1, 2],
    );
    let c = owned(&f, 0, &[1]);
    let r = report(&f, &c, 0, HtmlSemanticQuery::FormControls);
    assert_eq!(
        collection(&r),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::FormAssociation(f.ids[2].clone()))
    );
    assert_eq!(
        length(&report(&f, &c, 0, HtmlSemanticQuery::FormLength)),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::FormAssociation(f.ids[2].clone()))
    );
    assert!(r.usage().nested_output_records() > 0);
    let c = context(
        &f,
        &[
            (
                1,
                HtmlFormAssociationState::EstablishedCurrent(Some(f.ids[0].clone())),
            ),
            (2, HtmlFormAssociationState::EstablishedCurrent(None)),
        ],
        &[],
    );
    assert_eq!(
        controls(&report(&f, &c, 0, HtmlSemanticQuery::FormControls)),
        std::slice::from_ref(&f.ids[1])
    );
}

#[test]
fn image_proves_exclusion_before_unavailable_or_reset_form_attribute_evidence() {
    let f = flat(
        &[
            ("form", &[]),
            ("input", &[("type", "IMAGE"), ("form", "unread-oversized")]),
        ],
        &[0, 1],
    );
    for states in [
        vec![],
        vec![(1, HtmlFormAssociationState::Unavailable)],
        vec![(1, HtmlFormAssociationState::ResetAgainstCapture)],
    ] {
        let c = context(&f, &states, &[]);
        let r = session(&f, &c, limits(), 5, HtmlSemanticCancellation::new())
            .query(&[HtmlSemanticRequest::new(
                f.ids[0].clone(),
                HtmlSemanticQuery::FormControls,
            )])
            .unwrap();
        assert!(controls(&r).is_empty());
    }
}

#[test]
fn established_null_or_other_owner_avoids_oversized_input_type() {
    let f = flat(
        &[
            ("form", &[]),
            ("form", &[]),
            ("input", &[("type", "oversized")]),
        ],
        &[0, 1, 2],
    );
    for owner in [None, Some(f.ids[1].clone())] {
        let c = context(
            &f,
            &[(2, HtmlFormAssociationState::EstablishedCurrent(owner))],
            &[],
        );
        let r = session(&f, &c, limits(), 0, HtmlSemanticCancellation::new())
            .query(&[HtmlSemanticRequest::new(
                f.ids[0].clone(),
                HtmlSemanticQuery::FormControls,
            )])
            .unwrap();
        assert!(controls(&r).is_empty());
    }
}

#[test]
fn invalid_missing_and_hidden_input_types_keep_source_defaults_and_image_is_excluded() {
    for a in [
        &[][..],
        &[("type", "")][..],
        &[("type", "garbage")][..],
        &[("type", "hidden")][..],
        &[("type", "+image")][..],
        &[("type", " image")][..],
    ] {
        let f = flat(&[("form", &[]), ("input", a)], &[0, 1]);
        let c = owned(&f, 0, &[1]);
        assert_eq!(
            controls(&report(&f, &c, 0, HtmlSemanticQuery::FormControls)),
            std::slice::from_ref(&f.ids[1])
        );
    }
}

#[test]
fn original_consulted_type_and_reset_form_raw_require_exact_admission() {
    for (a, raw_length, state) in [
        (
            &[("type", "image")][..],
            5,
            HtmlFormAssociationState::Unavailable,
        ),
        (
            &[("form", "abcdef")][..],
            6,
            HtmlFormAssociationState::ResetAgainstCapture,
        ),
    ] {
        let f = flat(&[("form", &[("id", "abcdef")]), ("input", a)], &[0, 1]);
        let c = context(&f, &[(1, state)], &[]);
        let req = HtmlSemanticRequest::new(f.ids[0].clone(), HtmlSemanticQuery::FormControls);
        assert!(
            session(
                &f,
                &c,
                limits(),
                raw_length,
                HtmlSemanticCancellation::new()
            )
            .query(std::slice::from_ref(&req))
            .is_ok()
        );
        let mut s = session(
            &f,
            &c,
            limits(),
            raw_length - 1,
            HtmlSemanticCancellation::new(),
        );
        let e = s.query(std::slice::from_ref(&req)).unwrap_err();
        assert!(matches!(e.kind(), HtmlSemanticFailureKind::Attribute(_)));
        assert_eq!(s.query(&[]).unwrap_err(), e);
        assert_eq!(s.usage().scratch_records(), 0);
    }
}

#[test]
fn supplied_custom_true_false_and_missing_remain_distinct() {
    let f = flat(&[("form", &[]), ("x-control", &[])], &[0, 1]);
    let no = context(&f, &[], &[(1, false)]);
    assert!(controls(&report(&f, &no, 0, HtmlSemanticQuery::FormControls)).is_empty());
    let yes = context(
        &f,
        &[(
            1,
            HtmlFormAssociationState::EstablishedCurrent(Some(f.ids[0].clone())),
        )],
        &[(1, true)],
    );
    assert_eq!(
        controls(&report(&f, &yes, 0, HtmlSemanticQuery::FormControls)),
        std::slice::from_ref(&f.ids[1])
    );
    let unknown = context(&f, &[], &[]);
    assert_eq!(
        collection(&report(&f, &unknown, 0, HtmlSemanticQuery::FormControls)),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::CustomControlClassification(
            f.ids[1].clone()
        ))
    );
}

#[test]
fn established_other_or_null_owner_excludes_unknown_custom_without_inferred_classification() {
    let f = flat(
        &[("form", &[]), ("form", &[]), ("x-control", &[])],
        &[0, 2, 1],
    );
    for owner in [None, Some(f.ids[1].clone())] {
        let c = context(
            &f,
            &[(2, HtmlFormAssociationState::EstablishedCurrent(owner))],
            &[],
        );
        assert!(controls(&report(&f, &c, 0, HtmlSemanticQuery::FormControls)).is_empty());
        assert_eq!(
            length(&report(&f, &c, 0, HtmlSemanticQuery::FormLength)),
            &HtmlSemanticValue::Known(0)
        );
    }
    let c = context(
        &f,
        &[(
            2,
            HtmlFormAssociationState::EstablishedCurrent(Some(f.ids[0].clone())),
        )],
        &[],
    );
    assert_eq!(
        collection(&report(&f, &c, 0, HtmlSemanticQuery::FormControls)),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::CustomControlClassification(
            f.ids[2].clone()
        ))
    );
}

#[test]
fn fieldset_custom_membership_needs_classification_but_not_association() {
    let (mut b, d) = builder();
    let fieldset = element(&mut b, &d, "fieldset", &[]);
    let custom = element(&mut b, &d, "x-control", &[]);
    b.establish_children(&custom, &[]).unwrap();
    b.establish_children(&fieldset, std::slice::from_ref(&custom))
        .unwrap();
    b.establish_children(&d, std::slice::from_ref(&fieldset))
        .unwrap();
    let f = finish(b, &[fieldset, custom]);
    for (fact, expected) in [
        (Some(true), Some(true)),
        (Some(false), Some(false)),
        (None, None),
    ] {
        let c = context(&f, &[], &fact.map(|v| vec![(1, v)]).unwrap_or_default());
        let r = report(&f, &c, 0, HtmlSemanticQuery::FieldsetControls);
        match expected {
            Some(true) => assert_eq!(controls(&r), std::slice::from_ref(&f.ids[1])),
            Some(false) => assert!(controls(&r).is_empty()),
            None => assert_eq!(
                collection(&r),
                &HtmlSemanticValue::Missing(HtmlSemanticMissing::CustomControlClassification(
                    f.ids[1].clone()
                ))
            ),
        }
    }
}

#[test]
fn detached_complete_ordinary_tree_needs_no_connectedness_for_membership() {
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", &[]);
    let fieldset = element(&mut b, &d, "fieldset", &[]);
    let input = element(&mut b, &d, "input", &[]);
    b.establish_root(&form).unwrap();
    b.establish_children(&input, &[]).unwrap();
    b.establish_children(&fieldset, std::slice::from_ref(&input))
        .unwrap();
    b.establish_children(&form, std::slice::from_ref(&fieldset))
        .unwrap();
    let f = finish(b, &[form, fieldset, input]);
    let c = context(
        &f,
        &[
            (1, HtmlFormAssociationState::ResetAgainstCapture),
            (2, HtmlFormAssociationState::ResetAgainstCapture),
        ],
        &[],
    );
    assert_eq!(
        controls(&report(&f, &c, 0, HtmlSemanticQuery::FormControls)),
        &f.ids[1..]
    );
    assert_eq!(
        controls(&report(&f, &c, 1, HtmlSemanticQuery::FieldsetControls)),
        std::slice::from_ref(&f.ids[2])
    );
}

#[test]
fn listed_root_preserved_owner_can_match_descendant_form_and_unknown_is_not_null() {
    let (mut b, d) = builder();
    let root = element(&mut b, &d, "fieldset", &[]);
    let table = element(&mut b, &d, "table", &[]);
    let form = element(&mut b, &d, "form", &[]);
    let button = element(&mut b, &d, "button", &[]);
    b.establish_root(&root).unwrap();
    b.establish_children(&button, &[]).unwrap();
    b.establish_children(&form, std::slice::from_ref(&button))
        .unwrap();
    b.establish_children(&table, std::slice::from_ref(&form))
        .unwrap();
    b.establish_children(&root, std::slice::from_ref(&table))
        .unwrap();
    let f = finish(b, &[form, root, button]);
    // Source-reachable history: a malformed table/form parse can establish a
    // nonancestor owner; an atomic move preserving that owner can put its form
    // below the listed root. These literal final facts supply the actual pointer,
    // not an inferred nearest ancestor or an execution/history simulator.
    for (root_owner, expected) in [
        (
            Some(HtmlFormAssociationState::EstablishedCurrent(Some(
                f.ids[0].clone(),
            ))),
            Some(vec![f.ids[1].clone(), f.ids[2].clone()]),
        ),
        (
            Some(HtmlFormAssociationState::EstablishedCurrent(None)),
            Some(vec![f.ids[2].clone()]),
        ),
        (None, None),
    ] {
        let mut states = vec![(
            2,
            HtmlFormAssociationState::EstablishedCurrent(Some(f.ids[0].clone())),
        )];
        if let Some(owner) = root_owner {
            states.push((1, owner));
        }
        let c = context(&f, &states, &[]);
        let r = report(&f, &c, 0, HtmlSemanticQuery::FormControls);
        let count = report(&f, &c, 0, HtmlSemanticQuery::FormLength);
        if let Some(expected) = expected {
            assert_eq!(controls(&r), expected);
            assert_eq!(length(&count), &HtmlSemanticValue::Known(expected.len()));
        } else {
            let missing = HtmlSemanticMissing::FormAssociation(f.ids[1].clone());
            assert_eq!(collection(&r), &HtmlSemanticValue::Missing(missing.clone()));
            assert_eq!(length(&count), &HtmlSemanticValue::Missing(missing));
        }
    }
}

#[test]
fn form_parent_and_complete_children_are_distinct_required_evidence() {
    for root in [false, true] {
        let (mut b, d) = builder();
        let form = element(&mut b, &d, "form", &[]);
        if root {
            b.establish_root(&form).unwrap();
        }
        let f = finish(b, &[form]);
        let c = context(&f, &[], &[]);
        let missing = if root {
            HtmlSemanticMissing::OrderedChildren(f.ids[0].clone())
        } else {
            HtmlSemanticMissing::Parent(f.ids[0].clone())
        };
        assert_eq!(
            collection(&report(&f, &c, 0, HtmlSemanticQuery::FormControls)),
            &HtmlSemanticValue::Missing(missing)
        );
    }
}

#[test]
fn fieldset_requires_own_children_but_no_parent_and_late_unknown_subtree_blocks_full_set() {
    let (mut b, d) = builder();
    let fieldset = element(&mut b, &d, "fieldset", &[]);
    let input = element(&mut b, &d, "input", &[]);
    let late = element(&mut b, &d, "div", &[]);
    b.establish_children(&input, &[]).unwrap();
    b.establish_children(&fieldset, &[input.clone(), late.clone()])
        .unwrap();
    let f = finish(b, &[fieldset, input, late]);
    let c = context(&f, &[], &[]);
    let r = report(&f, &c, 0, HtmlSemanticQuery::FieldsetControls);
    assert_eq!(
        collection(&r),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(f.ids[2].clone()))
    );
    assert!(r.usage().nested_output_records() > 0);
    assert_eq!(r.usage().scratch_records(), 0);
}

#[test]
fn foreign_wrapper_is_traversed_but_foreign_or_uppercase_controls_are_not_native() {
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", &[]);
    let wrapper = expanded(&mut b, &d, Namespace::svg(), "svg", attrs(&[]));
    let foreign = expanded(&mut b, &d, Namespace::svg(), "input", attrs(&[]));
    let upper = element(&mut b, &d, "INPUT", &[]);
    let input = element(&mut b, &d, "input", &[]);
    for h in [&form, &foreign, &upper, &input] {
        b.establish_children(h, &[]).unwrap();
    }
    b.establish_children(&wrapper, &[foreign, upper.clone(), input.clone()])
        .unwrap();
    b.establish_children(&d, &[form.clone(), wrapper]).unwrap();
    let f = finish(b, &[form, input, upper]);
    let c = context(
        &f,
        &[(
            1,
            HtmlFormAssociationState::EstablishedCurrent(Some(f.ids[0].clone())),
        )],
        &[(2, false)],
    );
    assert_eq!(
        controls(&report(&f, &c, 0, HtmlSemanticQuery::FormControls)),
        std::slice::from_ref(&f.ids[1])
    );
}

#[test]
fn wrong_roles_non_elements_and_foreign_subjects_are_not_empty_collections() {
    let (mut b, d) = builder();
    let foreign = expanded(&mut b, &d, Namespace::svg(), "form", attrs(&[]));
    let wrong = element(&mut b, &d, "button", &[]);
    let f = finish(b, &[d, foreign, wrong]);
    let c = context(&f, &[], &[]);
    for q in [
        HtmlSemanticQuery::FormControls,
        HtmlSemanticQuery::FormLength,
        HtmlSemanticQuery::FieldsetControls,
    ] {
        for (target, expected) in [(0, false), (1, true), (2, false)] {
            let r = report(&f, &c, target, q);
            match r.results()[0].fact() {
                HtmlSemanticFact::FormLength(v) => {
                    if expected {
                        assert_eq!(
                            v,
                            &HtmlSemanticValue::Unsupported(
                                HtmlSemanticUnsupported::ForeignNamespace
                            )
                        );
                    } else {
                        assert_eq!(v, &HtmlSemanticValue::Inapplicable);
                    }
                }
                _ => {
                    if expected {
                        assert_eq!(
                            collection(&r),
                            &HtmlSemanticValue::Unsupported(
                                HtmlSemanticUnsupported::ForeignNamespace
                            )
                        );
                    } else {
                        assert_eq!(collection(&r), &HtmlSemanticValue::Inapplicable);
                    }
                }
            }
            assert_eq!(r.usage().nested_output_records(), 0);
            assert_eq!(r.usage().scratch_records(), 0);
        }
    }
}

#[test]
fn form_length_uses_same_membership_without_retained_member_capacity() {
    let f = flat(
        &[("form", &[]), ("input", &[]), ("button", &[])],
        &[1, 0, 2],
    );
    let c = owned(&f, 0, &[1, 2]);
    let l = HtmlSemanticLimits::new(10000000, 10, 1000, 0, 1000000, 0);
    let r = session(&f, &c, l, 0, HtmlSemanticCancellation::new())
        .query(&[HtmlSemanticRequest::new(
            f.ids[0].clone(),
            HtmlSemanticQuery::FormLength,
        )])
        .unwrap();
    assert_eq!(length(&r), &HtmlSemanticValue::Known(2));
    assert_eq!(r.usage().nested_output_records(), 0);
    assert_eq!(r.usage().scratch_records(), 0);
    let e = session(&f, &c, l, 0, HtmlSemanticCancellation::new())
        .query(&[HtmlSemanticRequest::new(
            f.ids[0].clone(),
            HtmlSemanticQuery::FormControls,
        )])
        .unwrap_err();
    assert!(matches!(
        e.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::NestedOutputRecords,
            ..
        }
    ));
}

#[test]
fn shadow_and_template_contents_are_not_ordinary_collection_descendants() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let form = element(&mut b, &d, "form", &[]);
        let fieldset = element(&mut b, &d, "fieldset", &[]);
        let host = element(&mut b, &d, if shadow { "div" } else { "template" }, &[]);
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
        let inside = element(&mut b, &d, "input", &[]);
        let outside = element(&mut b, &d, "input", &[]);
        for h in [&inside, &outside, &host] {
            b.establish_children(h, &[]).unwrap();
        }
        b.establish_children(&fragment, std::slice::from_ref(&inside))
            .unwrap();
        b.establish_children(&fieldset, &[host, outside.clone()])
            .unwrap();
        b.establish_children(&form, std::slice::from_ref(&fieldset))
            .unwrap();
        b.establish_children(&d, std::slice::from_ref(&form))
            .unwrap();
        let f = finish(b, &[form, fieldset, inside, outside]);
        let c = context(
            &f,
            &[
                (1, HtmlFormAssociationState::ResetAgainstCapture),
                (2, HtmlFormAssociationState::ResetAgainstCapture),
                (3, HtmlFormAssociationState::ResetAgainstCapture),
            ],
            &[],
        );
        assert_eq!(
            controls(&report(&f, &c, 0, HtmlSemanticQuery::FormControls)),
            &[f.ids[1].clone(), f.ids[3].clone()]
        );
        assert_eq!(
            controls(&report(&f, &c, 1, HtmlSemanticQuery::FieldsetControls)),
            std::slice::from_ref(&f.ids[3])
        );
    }
}

#[test]
fn reset_matching_preserves_exact_utf16_case_and_attribute_namespace() {
    for equal in [false, true] {
        let (mut b, d) = builder();
        let raw = DocumentString::from_utf16(vec![0xd800, 0, 65]);
        let other = DocumentString::from_utf16(vec![0xd800, 0, if equal { 65 } else { 97 }]);
        let form = expanded(
            &mut b,
            &d,
            Namespace::html(),
            "form",
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), None, "id"),
                raw.clone(),
            )])
            .unwrap(),
        );
        let input = expanded(
            &mut b,
            &d,
            Namespace::html(),
            "input",
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), Some("kept".into()), "form"),
                other,
            )])
            .unwrap(),
        );
        let namespaced = expanded(
            &mut b,
            &d,
            Namespace::html(),
            "input",
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::svg(), None, "form"),
                raw.clone(),
            )])
            .unwrap(),
        );
        for h in [&form, &input, &namespaced] {
            b.establish_children(h, &[]).unwrap();
        }
        b.establish_children(&d, &[input.clone(), form.clone(), namespaced.clone()])
            .unwrap();
        let f = finish(b, &[form, input, namespaced]);
        let c = context(
            &f,
            &[
                (1, HtmlFormAssociationState::ResetAgainstCapture),
                (2, HtmlFormAssociationState::ResetAgainstCapture),
            ],
            &[],
        );
        let r = session(&f, &c, limits(), 3, HtmlSemanticCancellation::new())
            .query(&[HtmlSemanticRequest::new(
                f.ids[0].clone(),
                HtmlSemanticQuery::FormControls,
            )])
            .unwrap();
        if equal {
            assert_eq!(controls(&r), std::slice::from_ref(&f.ids[1]));
        } else {
            assert!(controls(&r).is_empty());
        }
        let HtmlCapturedNodeKind::Element(e) = f.capture.node(&f.ids[0]).unwrap().kind() else {
            panic!()
        };
        assert_eq!(e.attributes().as_slice()[0].value(), &raw);
    }
}

fn capped(resource: HtmlSemanticResource, cap: usize) -> HtmlSemanticLimits {
    cap_resources(limits(), COLLECTION_RESOURCES, resource, cap)
}

#[test]
fn exact_and_one_short_work_output_scratch_nested_and_byte_caps_are_real() {
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", &[]);
    let fieldset = element(&mut b, &d, "fieldset", &[]);
    let input = element(&mut b, &d, "input", &[]);
    let button = element(&mut b, &d, "button", &[]);
    for h in [&input, &button] {
        b.establish_children(h, &[]).unwrap();
    }
    b.establish_children(&fieldset, &[input.clone(), button.clone()])
        .unwrap();
    b.establish_children(&form, std::slice::from_ref(&fieldset))
        .unwrap();
    b.establish_children(&d, std::slice::from_ref(&form))
        .unwrap();
    let f = finish(b, &[form, fieldset, input, button]);
    let c = owned(&f, 0, &[1, 2, 3]);
    for (target, q, count) in [
        (0, HtmlSemanticQuery::FormControls, 3),
        (0, HtmlSemanticQuery::FormLength, 3),
        (1, HtmlSemanticQuery::FieldsetControls, 2),
    ] {
        let req = HtmlSemanticRequest::new(f.ids[target].clone(), q);
        let baseline = report(&f, &c, target, q).usage();
        assert_eq!(baseline.scratch_records(), 0);
        assert!(baseline.peak_scratch_records() > 0);
        assert!(baseline.allocated_bytes() > 0);
        if q != HtmlSemanticQuery::FormLength {
            assert!(baseline.nested_output_records() >= count);
        }
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
                HtmlSemanticResource::NestedOutputRecords,
                baseline.nested_output_records(),
            ),
            (
                HtmlSemanticResource::AllocatedBytes,
                baseline.allocated_bytes(),
            ),
        ] {
            if exact == 0 {
                continue;
            }
            for cap in [exact, exact - 1] {
                let l = capped(resource, cap);
                let mut s = session(&f, &c, l, 0, HtmlSemanticCancellation::new());
                let result = s.query(std::slice::from_ref(&req));
                if cap == exact {
                    let r = result.unwrap();
                    if q == HtmlSemanticQuery::FormLength {
                        assert_eq!(length(&r), &HtmlSemanticValue::Known(count));
                    } else {
                        assert_eq!(controls(&r).len(), count);
                    }
                } else {
                    let failure = result.unwrap_err();
                    assert!(
                        matches!(failure.kind(), HtmlSemanticFailureKind::LimitExceeded { resource: r, .. } if *r == resource),
                        "{failure:?}"
                    );
                    let key = failure.qualification().unwrap();
                    assert_eq!(key.capture(), f.capture.identity());
                    assert_eq!(key.context(), c.identity());
                    assert_eq!(key.host_revision(), f.capture.host_facts_revision());
                    assert_eq!(key.limits(), l);
                    assert_eq!(s.query(&[]).unwrap_err(), failure);
                }
                assert_eq!(s.usage().scratch_records(), 0);
            }
        }
    }
}

#[test]
fn completed_collection_remains_valid_after_cancel_and_new_queries_stop_stickily() {
    let f = flat(&[("form", &[]), ("input", &[])], &[0, 1]);
    let c = owned(&f, 0, &[1]);
    let cancel = HtmlSemanticCancellation::new();
    let mut s = session(&f, &c, limits(), 0, cancel.clone());
    let req = HtmlSemanticRequest::new(f.ids[0].clone(), HtmlSemanticQuery::FormControls);
    let r = s.query(std::slice::from_ref(&req)).unwrap();
    cancel.cancel();
    let failure = s.query(&[req]).unwrap_err();
    assert_eq!(failure.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(s.query(&[]).unwrap_err(), failure);
    assert_eq!(controls(&r), std::slice::from_ref(&f.ids[1]));
    assert_eq!(s.usage().scratch_records(), 0);
}

#[test]
fn pre_cancelled_session_never_retains_any_collection_or_count() {
    let f = flat(&[("form", &[]), ("fieldset", &[])], &[0, 1]);
    let c = context(&f, &[], &[]);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut s = session(&f, &c, limits(), 0, cancel);
    let failure = s
        .query(&[HtmlSemanticRequest::new(
            f.ids[1].clone(),
            HtmlSemanticQuery::FieldsetControls,
        )])
        .unwrap_err();
    assert_eq!(failure.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(failure.qualification().unwrap().context(), c.identity());
    assert_eq!(failure.usage().nested_output_records(), 0);
    assert_eq!(failure.usage().work(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), failure);
}

#[test]
fn late_raw_failure_returns_no_successful_batch_prefix_and_cleans_scratch() {
    let f = flat(&[("form", &[]), ("input", &[("type", "image")])], &[0, 1]);
    let c = owned(&f, 0, &[1]);
    let mut s = session(&f, &c, limits(), 4, HtmlSemanticCancellation::new());
    let failure = s
        .query(&[
            HtmlSemanticRequest::new(f.ids[0].clone(), HtmlSemanticQuery::Hidden),
            HtmlSemanticRequest::new(f.ids[0].clone(), HtmlSemanticQuery::FormControls),
        ])
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    assert!(failure.usage().output_records() > 0);
    assert_eq!(s.usage().scratch_records(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), failure);
}

#[test]
fn duplicate_collection_requests_admit_cumulative_capacity_and_no_partial_batch() {
    let f = flat(&[("form", &[]), ("button", &[])], &[0, 1]);
    let c = owned(&f, 0, &[1]);
    let req = HtmlSemanticRequest::new(f.ids[0].clone(), HtmlSemanticQuery::FormControls);
    let usage = report(&f, &c, 0, HtmlSemanticQuery::FormControls).usage();
    let mut s = session(
        &f,
        &c,
        capped(
            HtmlSemanticResource::NestedOutputRecords,
            usage.nested_output_records(),
        ),
        0,
        HtmlSemanticCancellation::new(),
    );
    let e = s.query(&[req.clone(), req]).unwrap_err();
    assert!(matches!(
        e.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::NestedOutputRecords,
            ..
        }
    ));
    assert_eq!(
        e.usage().nested_output_records(),
        usage.nested_output_records()
    );
    assert_eq!(s.usage().scratch_records(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), e);
}

#[test]
fn changing_input_type_and_tree_order_changes_qualified_membership() {
    for (a, order, expected_count) in [
        (&[][..], &[2, 0, 1][..], 2),
        (&[("type", "image")][..], &[1, 0, 2][..], 1),
    ] {
        let f = flat(&[("form", &[]), ("input", a), ("button", &[])], order);
        let c = owned(&f, 0, &[1, 2]);
        let r = report(&f, &c, 0, HtmlSemanticQuery::FormControls);
        assert_eq!(controls(&r).len(), expected_count);
        assert_eq!(controls(&r)[0], f.ids[2]);
        assert_eq!(r.qualification().capture(), f.capture.identity());
        assert_eq!(
            r.qualification().host_revision(),
            f.capture.host_facts_revision()
        );
    }
}

#[test]
fn foreign_request_capture_and_mismatched_context_are_typed_and_qualified() {
    let f = flat(&[("form", &[])], &[0]);
    let other = flat(&[("form", &[])], &[0]);
    let c = context(&f, &[], &[]);
    let other_c = context(&other, &[], &[]);
    let e = HtmlSemanticSession::new(
        &f.capture,
        &other_c,
        HtmlValueInterpreter::new(0),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .err()
    .unwrap();
    assert_eq!(e.kind(), &HtmlSemanticFailureKind::ContextCaptureMismatch);
    assert!(e.qualification().is_none());
    let mut s = session(&f, &c, limits(), 0, HtmlSemanticCancellation::new());
    let e = s
        .query(&[HtmlSemanticRequest::new(
            other.ids[0].clone(),
            HtmlSemanticQuery::FormControls,
        )])
        .unwrap_err();
    assert!(matches!(
        e.kind(),
        HtmlSemanticFailureKind::Capture(HtmlCaptureError::ForeignCapture)
    ));
    assert_eq!(e.qualification().unwrap().capture(), f.capture.identity());
    assert_eq!(s.query(&[]).unwrap_err(), e);
}
