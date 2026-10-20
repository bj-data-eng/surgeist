use crate::semantics::support::{
    builder, collection_limits as capped, context, semantic_limits as limits, session,
};
use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlCapturedShadowProperties,
    HtmlMapNameAuthorReport, HtmlMapNameIdRelation, HtmlMapNameUniqueness,
    HtmlSemanticCancellation, HtmlSemanticDependencies, HtmlSemanticFact, HtmlSemanticFailureKind,
    HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport, HtmlSemanticRequest,
    HtmlSemanticResource, HtmlSemanticSession, HtmlSemanticUnsupported, HtmlSemanticValue,
};

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
    p: &HtmlCaptureBuilderSubject,
    cs: &[HtmlCaptureBuilderSubject],
) {
    b.establish_children(p, cs).unwrap();
}

fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::MapNameAuthor)
}
fn report(c: &HtmlCapture, id: &HtmlCaptureSubjectId, raw: usize) -> HtmlSemanticReport {
    session(
        c,
        &context(c),
        limits(),
        raw,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(id)])
    .unwrap()
}
fn value(r: &HtmlSemanticReport) -> &HtmlMapNameAuthorReport {
    match r.results()[0].fact() {
        HtmlSemanticFact::MapNameAuthor(HtmlSemanticValue::Known(v)) => v,
        other => panic!("{other:?}"),
    }
}
fn single(attrs: &[(&str, &str)]) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let m = html(&mut b, &d, "map", attrs);
    children(&mut b, &m, &[]);
    b.establish_root(&m).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&m).unwrap();
    (c, id)
}
fn pair(name: &str, other: &str) -> (HtmlCapture, HtmlCaptureSubjectId, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "div", &[]);
    let m = html(&mut b, &d, "map", &[("name", name)]);
    let n = html(&mut b, &d, "map", &[("name", other)]);
    children(&mut b, &m, &[]);
    children(&mut b, &n, &[]);
    children(&mut b, &root, &[m.clone(), n.clone()]);
    b.establish_root(&root).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&m).unwrap();
    let nid = c.subject_id(&n).unwrap();
    (c, id, nid)
}
#[test]
fn missing_name_skips_id_parent_children_and_unrelated_raw() {
    let (mut b, d) = builder();
    let m = html(
        &mut b,
        &d,
        "map",
        &[("id", "oversized"), ("coords", "huge")],
    );
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&m).unwrap(), 0);
    let v = value(&r);
    assert_eq!(v.local().author(), HtmlAttributeAuthorState::Missing);
    assert_eq!(v.id_attribute(), None);
    assert_eq!(v.id_relation(), HtmlMapNameIdRelation::NoName);
    assert_eq!(v.uniqueness(), &HtmlMapNameUniqueness::NoName);
    assert_eq!(r.usage().peak_scratch_records(), 0);
    assert_eq!(r.usage().nested_output_records(), 1);
}
#[test]
fn own_id_relation_does_not_validate_id_or_hide_local_failure() {
    for (name, id, relation) in [
        ("x", None, HtmlMapNameIdRelation::IdAbsent),
        ("x", Some("x"), HtmlMapNameIdRelation::Equal),
        ("x", Some("X"), HtmlMapNameIdRelation::Different),
        ("", Some(""), HtmlMapNameIdRelation::Equal),
        ("a b", Some("a b"), HtmlMapNameIdRelation::Equal),
    ] {
        let mut attrs = vec![("name", name)];
        if let Some(id) = id {
            attrs.push(("id", id));
        }
        let (c, m) = single(&attrs);
        let r = report(&c, &m, 3);
        let v = value(&r);
        assert_eq!(v.id_relation(), relation);
        assert_eq!(
            v.id_attribute().map(Attribute::value),
            id.map(DocumentString::from).as_ref()
        );
        assert_eq!(v.uniqueness(), &HtmlMapNameUniqueness::Unique);
        assert_eq!(
            v.local().context_author(),
            HtmlAttributeContextAuthorState::Unchecked
        );
        assert_eq!(
            v.local().source_clause(),
            HtmlSemanticQuery::MapNameAuthor.source_clause()
        );
        if name.is_empty() || name.contains(' ') {
            assert!(matches!(
                v.local().author(),
                HtmlAttributeAuthorState::Nonconforming(_)
            ));
        } else {
            assert_eq!(v.local().author(), HtmlAttributeAuthorState::Conforming);
        }
    }
}
#[test]
fn duplicates_compare_specified_values_even_when_local_grammar_is_invalid() {
    for name in ["", "a b", "x"] {
        let (c, m, n) = pair(name, name);
        let r = report(&c, &m, 3);
        assert_eq!(value(&r).uniqueness(), &HtmlMapNameUniqueness::Duplicate(n));
    }
}
#[test]
fn equality_is_case_sensitive_and_has_no_unicode_normalization() {
    for (a, b) in [("x", "X"), ("é", "e\u{301}"), ("x", " x"), ("x", "x\u{A0}")] {
        let (c, m, _) = pair(a, b);
        assert_eq!(
            value(&report(&c, &m, 4)).uniqueness(),
            &HtmlMapNameUniqueness::Unique
        );
    }
}
#[test]
fn exact_surrogate_values_and_id_equality_are_preserved() {
    let (mut b, d) = builder();
    let units = DocumentString::from_utf16(vec![0xD800, 0, 0xDC00]);
    let attrs = OrderedAttributes::new([
        Attribute::new(KnownHtmlAttribute::Name.qualified_name(), units.clone()),
        Attribute::new(KnownHtmlAttribute::Id.qualified_name(), units.clone()),
    ])
    .unwrap();
    let m = element(&mut b, &d, KnownHtmlElement::Map.qualified_name(), attrs);
    children(&mut b, &m, &[]);
    b.establish_root(&m).unwrap();
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&m).unwrap(), 3);
    assert_eq!(value(&r).local().intrinsic().unwrap().value(), &units);
    assert_eq!(value(&r).id_relation(), HtmlMapNameIdRelation::Equal);
}
#[test]
fn only_other_exact_html_maps_with_specified_name_participate() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "div", &[]);
    let m = html(&mut b, &d, "map", &[("name", "x")]);
    let mut nodes = vec![m.clone()];
    for (ns, local, attrs) in [
        (Namespace::html(), "MAP", vec![("name", "x")]),
        (Namespace::svg(), "map", vec![("name", "x")]),
        (Namespace::html(), "span", vec![("name", "x")]),
        (Namespace::html(), "map", vec![("id", "x")]),
    ] {
        let n =
            element(
                &mut b,
                &d,
                QualifiedName::new(ns, None, local),
                OrderedAttributes::new(attrs.iter().map(|(n, v)| {
                    Attribute::new(QualifiedName::new(Namespace::none(), None, *n), *v)
                }))
                .unwrap(),
            );
        children(&mut b, &n, &[]);
        nodes.push(n);
    }
    children(&mut b, &m, &[]);
    children(&mut b, &root, &nodes);
    b.establish_root(&root).unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&m).unwrap(), 1)).uniqueness(),
        &HtmlMapNameUniqueness::Unique
    );
}
#[test]
fn foreign_wrappers_do_not_prune_ordinary_html_map_descendants() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "map", &[("name", "x")]);
    let wrapper = element(
        &mut b,
        &d,
        QualifiedName::new(Namespace::svg(), None, "g"),
        OrderedAttributes::new([]).unwrap(),
    );
    let duplicate = html(&mut b, &d, "map", &[("name", "x")]);
    children(&mut b, &wrapper, std::slice::from_ref(&duplicate));
    children(&mut b, &root, &[wrapper]);
    b.establish_root(&root).unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&root).unwrap(), 1)).uniqueness(),
        &HtmlMapNameUniqueness::Duplicate(c.subject_id(&duplicate).unwrap())
    );
}
#[test]
fn partial_parent_component_can_prove_duplicate_but_cannot_prove_unique() {
    for duplicate in [false, true] {
        let (mut b, d) = builder();
        let root = html(&mut b, &d, "div", &[]);
        let m = html(&mut b, &d, "map", &[("name", "x")]);
        let n = html(
            &mut b,
            &d,
            "map",
            &[("name", if duplicate { "x" } else { "y" })],
        );
        children(&mut b, &m, &[]);
        children(&mut b, &n, &[]);
        children(&mut b, &root, &[m.clone(), n.clone()]);
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&m).unwrap(), 1);
        assert_eq!(
            value(&r).uniqueness(),
            &if duplicate {
                HtmlMapNameUniqueness::Duplicate(c.subject_id(&n).unwrap())
            } else {
                HtmlMapNameUniqueness::Missing(HtmlSemanticMissing::Parent(
                    c.subject_id(&root).unwrap(),
                ))
            }
        );
        assert_eq!(r.usage().scratch_records(), 0);
    }
}
#[test]
fn unavailable_gap_does_not_hide_later_duplicate_or_fabricate_uniqueness() {
    for duplicate in [false, true] {
        let (mut b, d) = builder();
        let root = html(&mut b, &d, "div", &[]);
        let m = html(&mut b, &d, "map", &[("name", "x")]);
        let gap = html(&mut b, &d, "div", &[]);
        let n = html(
            &mut b,
            &d,
            "map",
            &[("name", if duplicate { "x" } else { "y" })],
        );
        children(&mut b, &m, &[]);
        children(&mut b, &n, &[]);
        children(&mut b, &root, &[m.clone(), gap.clone(), n.clone()]);
        b.establish_root(&root).unwrap();
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&m).unwrap(), 1);
        assert_eq!(
            value(&r).uniqueness(),
            &if duplicate {
                HtmlMapNameUniqueness::Duplicate(c.subject_id(&n).unwrap())
            } else {
                HtmlMapNameUniqueness::Missing(HtmlSemanticMissing::OrderedChildren(
                    c.subject_id(&gap).unwrap(),
                ))
            }
        );
        assert_eq!(r.usage().scratch_records(), 0);
    }
}
#[test]
fn duplicate_witness_skips_its_unknown_children_and_later_oversized_names() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "div", &[]);
    let m = html(&mut b, &d, "map", &[("name", "x")]);
    let n = html(&mut b, &d, "map", &[("name", "x")]);
    let later = html(&mut b, &d, "map", &[("name", "oversized")]);
    children(&mut b, &m, &[]);
    children(&mut b, &root, &[m.clone(), n.clone(), later]);
    b.establish_root(&root).unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&m).unwrap(), 1)).uniqueness(),
        &HtmlMapNameUniqueness::Duplicate(c.subject_id(&n).unwrap())
    );
}
#[test]
fn consulted_own_id_and_competing_name_caps_are_terminal_before_comparison() {
    for own in [true, false] {
        let (mut b, d) = builder();
        let root = html(&mut b, &d, "div", &[]);
        let m = html(
            &mut b,
            &d,
            "map",
            if own {
                &[("name", "x"), ("id", "oversized")]
            } else {
                &[("name", "x")]
            },
        );
        let earlier = html(&mut b, &d, "map", &[("name", "oversized")]);
        let duplicate = html(&mut b, &d, "map", &[("name", "x")]);
        children(&mut b, &m, &[]);
        children(&mut b, &earlier, &[]);
        children(&mut b, &root, &[m.clone(), earlier, duplicate]);
        b.establish_root(&root).unwrap();
        let c = b.finish().unwrap();
        let cx = context(&c);
        let mut s = session(&c, &cx, limits(), 1, HtmlSemanticCancellation::new());
        let error = s.query(&[request(&c.subject_id(&m).unwrap())]).unwrap_err();
        assert!(matches!(
            error.kind(),
            HtmlSemanticFailureKind::Attribute(_)
        ));
        assert_eq!(s.query(&[]).unwrap_err(), error);
        assert_eq!(s.usage().scratch_records(), 0);
    }
}
#[test]
fn wrong_roles_and_non_element_subjects_are_distinct_from_foreign_namespace() {
    let (mut b, d) = builder();
    let wrong = html(&mut b, &d, "MAP", &[("name", "huge")]);
    let foreign = element(
        &mut b,
        &d,
        QualifiedName::new(Namespace::svg(), None, "map"),
        OrderedAttributes::new([]).unwrap(),
    );
    let text = b.add_text(&d, "raw".into()).unwrap();
    let c = b.finish().unwrap();
    for subject in [wrong, text, d] {
        assert_eq!(
            report(&c, &c.subject_id(&subject).unwrap(), 0).results()[0].fact(),
            &HtmlSemanticFact::MapNameAuthor(HtmlSemanticValue::Inapplicable)
        );
    }
    assert_eq!(
        report(&c, &c.subject_id(&foreign).unwrap(), 0).results()[0].fact(),
        &HtmlSemanticFact::MapNameAuthor(HtmlSemanticValue::Unsupported(
            HtmlSemanticUnsupported::ForeignNamespace
        ))
    );
}

#[test]
fn exact_and_one_short_caps_account_for_single_record_and_actual_scratch_storage() {
    let (c, m, _) = pair("x", "y");
    let cx = context(&c);
    let observed = report(&c, &m, 1).usage();
    eprintln!("complete pair map-name cost: {observed:?}");
    assert_eq!(observed.nested_output_records(), 1);
    assert_eq!(observed.scratch_records(), 0);
    for (resource, exact) in [
        (HtmlSemanticResource::Work, observed.work()),
        (
            HtmlSemanticResource::OutputRecords,
            observed.output_records(),
        ),
        (
            HtmlSemanticResource::ScratchRecords,
            observed.peak_scratch_records(),
        ),
        (
            HtmlSemanticResource::NestedOutputRecords,
            observed.nested_output_records(),
        ),
        (
            HtmlSemanticResource::AllocatedBytes,
            observed.allocated_bytes(),
        ),
    ] {
        assert!(exact > 0);
        for cap in [exact, exact - 1] {
            let mut s = session(
                &c,
                &cx,
                capped(resource, cap),
                1,
                HtmlSemanticCancellation::new(),
            );
            let result = s.query(&[request(&m)]);
            if cap == exact {
                assert_eq!(
                    value(&result.unwrap()).uniqueness(),
                    &HtmlMapNameUniqueness::Unique
                );
            } else {
                let failure = result.unwrap_err();
                assert!(
                    matches!(failure.kind(),HtmlSemanticFailureKind::LimitExceeded {resource:r,..} if *r==resource),
                    "{failure:?}"
                );
                assert_eq!(s.query(&[]).unwrap_err(), failure);
            }
            assert_eq!(s.usage().scratch_records(), 0);
        }
    }
}
#[test]
fn raw_exact_and_one_short_length_are_independent_of_physical_budgets() {
    let (c, m) = single(&[("name", "abc")]);
    assert_eq!(
        value(&report(&c, &m, 3)).uniqueness(),
        &HtmlMapNameUniqueness::Unique
    );
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 2, HtmlSemanticCancellation::new());
    assert!(matches!(
        s.query(&[request(&m)]).unwrap_err().kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
}
#[test]
fn repeated_queries_charge_all_retained_records_and_late_batch_failure_is_atomic() {
    let (c, m, _) = pair("x", "y");
    let cx = context(&c);
    let one = report(&c, &m, 1).usage();
    let mut s = session(&c, &cx, limits(), 1, HtmlSemanticCancellation::new());
    let twice = s.query(&[request(&m), request(&m)]).unwrap();
    assert_eq!(
        twice.usage().nested_output_records(),
        2 * one.nested_output_records()
    );
    assert!(twice.usage().allocated_bytes() >= 2 * one.allocated_bytes());
    for resource in [
        HtmlSemanticResource::NestedOutputRecords,
        HtmlSemanticResource::OutputRecords,
    ] {
        let cap = if resource == HtmlSemanticResource::NestedOutputRecords {
            one.nested_output_records()
        } else {
            1
        };
        let mut s = session(
            &c,
            &cx,
            capped(resource, cap),
            1,
            HtmlSemanticCancellation::new(),
        );
        let failure = s.query(&[request(&m), request(&m)]).unwrap_err();
        assert!(
            matches!(failure.kind(),HtmlSemanticFailureKind::LimitExceeded {resource:r,..} if *r==resource)
        );
        assert_eq!(failure.qualification().unwrap().capture(), c.identity());
        assert_eq!(failure.qualification().unwrap().context(), cx.identity());
        assert_eq!(s.usage().scratch_records(), 0);
        assert_eq!(s.query(&[]).unwrap_err(), failure);
    }
}
#[test]
fn cancellation_is_sticky_qualified_and_admits_no_partial_result() {
    let (c, m) = single(&[("name", "x")]);
    let cx = context(&c);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut s = session(&c, &cx, limits(), 1, cancel);
    let failure = s.query(&[request(&m)]).unwrap_err();
    assert_eq!(failure.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(s.query(&[]).unwrap_err(), failure);
    assert_eq!(s.usage().scratch_records(), 0);
    assert_eq!(failure.qualification().unwrap().capture(), c.identity());
}
#[test]
fn immutable_results_are_capture_context_host_and_configuration_qualified() {
    let (a, m, _) = pair("x", "y");
    let (b, n, duplicate) = pair("x", "x");
    let ar = report(&a, &m, 1);
    let br = report(&b, &n, 1);
    assert_eq!(value(&ar).uniqueness(), &HtmlMapNameUniqueness::Unique);
    assert_eq!(
        value(&br).uniqueness(),
        &HtmlMapNameUniqueness::Duplicate(duplicate)
    );
    assert_ne!(ar.capture(), br.capture());
    assert_ne!(ar.host_revision(), br.host_revision());
    assert_eq!(
        ar.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
    let cx = context(&a);
    let mut s = session(
        &a,
        &cx,
        capped(HtmlSemanticResource::Work, ar.usage().work()),
        1,
        HtmlSemanticCancellation::new(),
    );
    let cr = s.query(&[request(&m)]).unwrap();
    assert_ne!(cr.context(), ar.context());
    assert_ne!(cr.qualification(), ar.qualification());
    assert_eq!(value(&ar).uniqueness(), &HtmlMapNameUniqueness::Unique);
}
#[test]
fn foreign_capture_subject_is_a_terminal_qualified_rejection() {
    let (a, _, _) = pair("x", "y");
    let (b, n, _) = pair("x", "x");
    let cx = context(&a);
    let mut s = session(&a, &cx, limits(), 1, HtmlSemanticCancellation::new());
    let error = s.query(&[request(&n)]).unwrap_err();
    assert!(matches!(error.kind(), HtmlSemanticFailureKind::Capture(_)));
    assert_eq!(error.qualification().unwrap().capture(), a.identity());
    assert!(
        HtmlSemanticSession::new(
            &a,
            &context(&b),
            HtmlValueInterpreter::new(1),
            limits(),
            HtmlSemanticCancellation::new()
        )
        .is_err()
    );
}

#[test]
fn shadow_and_template_host_associations_are_not_ordinary_tree_edges() {
    for shadow in [true, false] {
        let (mut b, d) = builder();
        let m = html(&mut b, &d, "map", &[("name", "x")]);
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
        let hidden = html(&mut b, &d, "map", &[("name", "x")]);
        children(&mut b, &hidden, &[]);
        children(&mut b, &f, std::slice::from_ref(&hidden));
        children(&mut b, &host, &[]);
        children(&mut b, &m, &[host]);
        b.establish_root(&m).unwrap();
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&m).unwrap(), 1)).uniqueness(),
            &HtmlMapNameUniqueness::Unique
        );
        assert_eq!(
            value(&report(&c, &c.subject_id(&hidden).unwrap(), 1)).uniqueness(),
            &HtmlMapNameUniqueness::Unique
        );
    }
}
#[test]
fn namespaced_name_and_id_attributes_are_not_consulted() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "div", &[]);
    let m = element(
        &mut b,
        &d,
        KnownHtmlElement::Map.qualified_name(),
        OrderedAttributes::new([
            Attribute::new(KnownHtmlAttribute::Name.qualified_name(), "x"),
            Attribute::new(
                QualifiedName::new(Namespace::svg(), None, "id"),
                "oversized",
            ),
        ])
        .unwrap(),
    );
    let n = element(
        &mut b,
        &d,
        KnownHtmlElement::Map.qualified_name(),
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::svg(), None, "name"),
            "oversized",
        )])
        .unwrap(),
    );
    children(&mut b, &m, &[]);
    children(&mut b, &n, &[]);
    children(&mut b, &root, &[m.clone(), n]);
    b.establish_root(&root).unwrap();
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&m).unwrap(), 1);
    assert_eq!(value(&r).id_relation(), HtmlMapNameIdRelation::IdAbsent);
    assert_eq!(value(&r).uniqueness(), &HtmlMapNameUniqueness::Unique);
}
#[test]
fn known_id_mismatch_remains_delivered_with_incomplete_uniqueness() {
    let (mut b, d) = builder();
    let m = html(&mut b, &d, "map", &[("name", "x"), ("id", "y")]);
    b.establish_root(&m).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&m).unwrap();
    let r = report(&c, &id, 1);
    assert_eq!(value(&r).id_relation(), HtmlMapNameIdRelation::Different);
    assert_eq!(
        value(&r).uniqueness(),
        &HtmlMapNameUniqueness::Missing(HtmlSemanticMissing::OrderedChildren(id))
    );
}
#[test]
fn unavailable_gap_does_not_hide_terminal_work_or_scratch_admission() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "div", &[]);
    let m = html(&mut b, &d, "map", &[("name", "x")]);
    let gap = html(&mut b, &d, "div", &[]);
    let n = html(&mut b, &d, "map", &[("name", "x")]);
    children(&mut b, &m, &[]);
    children(&mut b, &root, &[m.clone(), gap, n]);
    b.establish_root(&root).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&m).unwrap();
    let cx = context(&c);
    let u = report(&c, &id, 1).usage();
    for (resource, cap) in [
        (HtmlSemanticResource::Work, u.work() - 1),
        (HtmlSemanticResource::ScratchRecords, 0),
    ] {
        let mut s = session(
            &c,
            &cx,
            capped(resource, cap),
            1,
            HtmlSemanticCancellation::new(),
        );
        let error = s.query(&[request(&id)]).unwrap_err();
        assert!(
            matches!(error.kind(),HtmlSemanticFailureKind::LimitExceeded {resource:r,..} if *r==resource)
        );
        assert_eq!(s.query(&[]).unwrap_err(), error);
        assert_eq!(s.usage().scratch_records(), 0);
    }
}
#[test]
fn late_raw_failure_discards_an_earlier_complete_requested_result() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "div", &[]);
    let valid = html(&mut b, &d, "map", &[]);
    let invalid = html(&mut b, &d, "map", &[("name", "oversized")]);
    children(&mut b, &root, &[valid.clone(), invalid.clone()]);
    b.establish_root(&root).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let error = s
        .query(&[
            request(&c.subject_id(&valid).unwrap()),
            request(&c.subject_id(&invalid).unwrap()),
        ])
        .unwrap_err();
    assert!(matches!(
        error.kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    assert_eq!(s.usage().nested_output_records(), 1);
    assert_eq!(s.usage().scratch_records(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), error);
}
