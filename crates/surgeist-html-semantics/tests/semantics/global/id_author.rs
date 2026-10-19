use crate::semantics::support::builder;
use crate::semantics::support::collection_limits as capped;
use crate::semantics::support::context;
use crate::semantics::support::session;
use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlCapturedShadowProperties, HtmlIdAuthorReport,
    HtmlIdUniqueness, HtmlSemanticCancellation, HtmlSemanticDependencies, HtmlSemanticFact,
    HtmlSemanticFailureKind, HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery,
    HtmlSemanticReport, HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticSession,
    HtmlSemanticUnsupported, HtmlSemanticValue,
};

// Literal expectations follow HTML 877d #the-id-attribute and the retained
// DOM #concept-id: only nonempty no-namespace lowercase id is a DOM ID, on
// elements of every namespace. Author syntax and tree uniqueness are separate.

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
    id: Option<&str>,
) -> HtmlCaptureBuilderSubject {
    element(
        b,
        d,
        QualifiedName::new(Namespace::html(), None, local),
        OrderedAttributes::new(
            id.map(|v| Attribute::new(KnownHtmlAttribute::Id.qualified_name(), v)),
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
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::IdAuthor)
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
fn value(r: &HtmlSemanticReport) -> &HtmlIdAuthorReport {
    match r.results()[0].fact() {
        HtmlSemanticFact::IdAuthor(HtmlSemanticValue::Known(v)) => v,
        other => panic!("{other:?}"),
    }
}
fn single(id: Option<&str>) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let n = html(&mut b, &d, "x-authored", id);
    children(&mut b, &n, &[]);
    b.establish_root(&n).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&n).unwrap();
    (c, id)
}
fn pair(a: &str, z: &str) -> (HtmlCapture, HtmlCaptureSubjectId, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "div", None);
    let n = html(&mut b, &d, "p", Some(a));
    let other = html(&mut b, &d, "span", Some(z));
    children(&mut b, &n, &[]);
    children(&mut b, &other, &[]);
    children(&mut b, &root, &[n.clone(), other.clone()]);
    b.establish_root(&root).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&n).unwrap();
    let oid = c.subject_id(&other).unwrap();
    (c, id, oid)
}

#[test]
fn missing_and_empty_ids_preserve_local_states_without_consulting_unknown_topology() {
    for id in [None, Some("")] {
        let (mut b, d) = builder();
        let mut attrs = vec![Attribute::new(
            KnownHtmlAttribute::Title.qualified_name(),
            "oversized",
        )];
        if let Some(id) = id {
            attrs.push(Attribute::new(KnownHtmlAttribute::Id.qualified_name(), id));
        }
        let n = element(
            &mut b,
            &d,
            KnownHtmlElement::Div.qualified_name(),
            OrderedAttributes::new(attrs).unwrap(),
        );
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&n).unwrap(), 0);
        let v = value(&r);
        assert_eq!(v.uniqueness(), &HtmlIdUniqueness::NoId);
        if id.is_none() {
            assert_eq!(v.local().author(), HtmlAttributeAuthorState::Missing);
            assert_eq!(
                v.local().context_author(),
                HtmlAttributeContextAuthorState::NotRequired
            );
            assert_eq!(v.local().raw(), None);
        } else {
            assert_eq!(v.local().recognition(), HtmlAttributeRecognition::Empty);
            assert!(matches!(
                v.local().author(),
                HtmlAttributeAuthorState::Nonconforming(_)
            ));
            assert_eq!(
                v.local().context_author(),
                HtmlAttributeContextAuthorState::Unchecked
            );
            assert!(v.local().raw().unwrap().is_empty());
        }
        assert_eq!(r.usage().peak_scratch_records(), 0);
    }
}

#[test]
fn a_complete_detached_custom_html_element_can_have_a_unique_id() {
    let (c, id) = single(Some("7!opaque"));
    let r = report(&c, &id, 8);
    let v = value(&r);
    assert_eq!(v.uniqueness(), &HtmlIdUniqueness::Unique);
    assert_eq!(v.local().author(), HtmlAttributeAuthorState::Conforming);
    assert_eq!(
        v.local().context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
    assert_eq!(
        v.local().intrinsic().unwrap().value(),
        &DocumentString::from("7!opaque")
    );
    assert_eq!(
        v.local().source_clause(),
        HtmlSemanticQuery::IdAuthor.source_clause()
    );
    assert_eq!(r.usage().nested_output_records(), 1);
    assert_eq!(r.usage().scratch_records(), 0);
}

#[test]
fn competing_ids_on_any_element_namespace_are_duplicates() {
    for namespace in [
        Namespace::svg(),
        Namespace::mathml(),
        Namespace::none(),
        Namespace::html(),
        Namespace::from_uri("urn:unknown"),
    ] {
        let (mut b, d) = builder();
        let root = html(&mut b, &d, "div", None);
        let own = html(&mut b, &d, "p", Some("x"));
        let other = element(
            &mut b,
            &d,
            QualifiedName::new(namespace, None, "FOREIGN"),
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), Some("kept".into()), "id"),
                "x",
            )])
            .unwrap(),
        );
        children(&mut b, &own, &[]);
        children(&mut b, &root, &[own.clone(), other.clone()]);
        b.establish_root(&root).unwrap();
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&own).unwrap(), 1)).uniqueness(),
            &HtmlIdUniqueness::Duplicate(c.subject_id(&other).unwrap())
        );
    }
}

#[test]
fn namespaced_uppercase_empty_and_name_only_slots_are_not_dom_ids() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "div", None);
    let own = html(&mut b, &d, "p", Some("x"));
    let mut nodes = vec![own.clone()];
    for attr in [
        Attribute::new(
            QualifiedName::new(Namespace::svg(), None, "id"),
            "oversized",
        ),
        Attribute::new(
            QualifiedName::new(Namespace::none(), None, "ID"),
            "oversized",
        ),
        Attribute::new(KnownHtmlAttribute::Id.qualified_name(), ""),
        Attribute::new(KnownHtmlAttribute::Name.qualified_name(), "oversized"),
    ] {
        let n = element(
            &mut b,
            &d,
            KnownHtmlElement::Span.qualified_name(),
            OrderedAttributes::new([attr]).unwrap(),
        );
        children(&mut b, &n, &[]);
        nodes.push(n);
    }
    children(&mut b, &own, &[]);
    children(&mut b, &root, &nodes);
    b.establish_root(&root).unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&own).unwrap(), 1)).uniqueness(),
        &HtmlIdUniqueness::Unique
    );
}

#[test]
fn invalid_nonempty_html_ids_still_compete_as_exact_dom_ids() {
    for id in ["a b", "\t", "\r\n", "x\u{c}"] {
        let (c, own, other) = pair(id, id);
        let r = report(&c, &own, 4);
        let v = value(&r);
        assert!(matches!(
            v.local().author(),
            HtmlAttributeAuthorState::Nonconforming(_)
        ));
        assert_eq!(v.local().raw(), Some(&DocumentString::from(id)));
        assert_eq!(v.uniqueness(), &HtmlIdUniqueness::Duplicate(other));
    }
}

#[test]
fn id_comparison_is_case_sensitive_and_does_not_trim_or_normalize() {
    for (a, z) in [("x", "X"), ("x", " x"), ("é", "e\u{301}"), ("x", "x\u{a0}")] {
        let (c, own, _) = pair(a, z);
        assert_eq!(
            value(&report(&c, &own, 4)).uniqueness(),
            &HtmlIdUniqueness::Unique
        );
    }
}

#[test]
fn null_and_isolated_surrogates_preserve_exact_identity_and_raw_backing() {
    for other in [vec![0xd800, 0, 0xdc00], vec![0xd800, 0, 0xfffd]] {
        let (mut b, d) = builder();
        let root = html(&mut b, &d, "div", None);
        let raw = DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]);
        let own = element(
            &mut b,
            &d,
            KnownHtmlElement::P.qualified_name(),
            OrderedAttributes::new([Attribute::new(
                KnownHtmlAttribute::Id.qualified_name(),
                raw.clone(),
            )])
            .unwrap(),
        );
        let duplicate = other == [0xd800, 0, 0xdc00];
        let n = element(
            &mut b,
            &d,
            QualifiedName::new(Namespace::svg(), None, "g"),
            OrderedAttributes::new([Attribute::new(
                KnownHtmlAttribute::Id.qualified_name(),
                DocumentString::from_utf16(other),
            )])
            .unwrap(),
        );
        children(&mut b, &own, &[]);
        children(&mut b, &n, &[]);
        children(&mut b, &root, &[own.clone(), n.clone()]);
        b.establish_root(&root).unwrap();
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&own).unwrap(), 3);
        let v = value(&r);
        assert_eq!(v.local().raw(), Some(&raw));
        assert_eq!(v.local().intrinsic().unwrap().value(), &raw);
        assert_eq!(
            v.uniqueness(),
            &if duplicate {
                HtmlIdUniqueness::Duplicate(c.subject_id(&n).unwrap())
            } else {
                HtmlIdUniqueness::Unique
            }
        );
    }
}

#[test]
fn ordinary_root_ids_participate_and_other_detached_roots_do_not() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "div", Some("x"));
    let own = html(&mut b, &d, "p", Some("x"));
    let detached = html(&mut b, &d, "p", Some("x"));
    children(&mut b, &detached, &[]);
    b.establish_root(&detached).unwrap();
    children(&mut b, &own, &[]);
    children(&mut b, &root, std::slice::from_ref(&own));
    b.establish_root(&root).unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&own).unwrap(), 1)).uniqueness(),
        &HtmlIdUniqueness::Duplicate(c.subject_id(&root).unwrap())
    );
    assert_eq!(
        value(&report(&c, &c.subject_id(&detached).unwrap(), 1)).uniqueness(),
        &HtmlIdUniqueness::Unique
    );
}

#[test]
fn unknown_ancestry_preserves_missing_unless_a_known_subtree_proves_duplicate() {
    for duplicate in [false, true] {
        let (mut b, d) = builder();
        let own = html(&mut b, &d, "div", Some("x"));
        let n = html(&mut b, &d, "p", Some(if duplicate { "x" } else { "y" }));
        children(&mut b, &n, &[]);
        children(&mut b, &own, std::slice::from_ref(&n));
        let c = b.finish().unwrap();
        let id = c.subject_id(&own).unwrap();
        assert_eq!(
            value(&report(&c, &id, 1)).uniqueness(),
            &if duplicate {
                HtmlIdUniqueness::Duplicate(c.subject_id(&n).unwrap())
            } else {
                HtmlIdUniqueness::Missing(HtmlSemanticMissing::Parent(id))
            }
        );
    }
}

#[test]
fn unknown_children_do_not_certify_unique_or_hide_a_later_duplicate() {
    for duplicate in [false, true] {
        let (mut b, d) = builder();
        let root = html(&mut b, &d, "div", None);
        let own = html(&mut b, &d, "p", Some("x"));
        let gap = html(&mut b, &d, "span", None);
        let n = html(&mut b, &d, "span", Some(if duplicate { "x" } else { "y" }));
        children(&mut b, &own, &[]);
        children(&mut b, &n, &[]);
        children(&mut b, &root, &[own.clone(), gap.clone(), n.clone()]);
        b.establish_root(&root).unwrap();
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&own).unwrap(), 1);
        assert_eq!(
            value(&r).uniqueness(),
            &if duplicate {
                HtmlIdUniqueness::Duplicate(c.subject_id(&n).unwrap())
            } else {
                HtmlIdUniqueness::Missing(HtmlSemanticMissing::OrderedChildren(
                    c.subject_id(&gap).unwrap(),
                ))
            }
        );
        assert_eq!(r.usage().scratch_records(), 0);
    }
}

#[test]
fn known_duplicate_skips_its_children_and_later_oversized_values() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "div", None);
    let own = html(&mut b, &d, "p", Some("x"));
    let witness = html(&mut b, &d, "p", Some("x"));
    let later = html(&mut b, &d, "p", Some("oversized"));
    children(&mut b, &own, &[]);
    children(&mut b, &root, &[own.clone(), witness.clone(), later]);
    b.establish_root(&root).unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&own).unwrap(), 1)).uniqueness(),
        &HtmlIdUniqueness::Duplicate(c.subject_id(&witness).unwrap())
    );
}

#[test]
fn actual_fragment_children_count_without_crossing_template_or_shadow_associations() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let own = html(&mut b, &d, "div", Some("x"));
        let host = html(&mut b, &d, if shadow { "div" } else { "template" }, None);
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
        let inner = html(&mut b, &d, "p", Some("x"));
        children(&mut b, &inner, &[]);
        children(&mut b, &fragment, std::slice::from_ref(&inner));
        children(&mut b, &host, &[]);
        children(&mut b, &own, &[host]);
        b.establish_root(&own).unwrap();
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&own).unwrap(), 1)).uniqueness(),
            &HtmlIdUniqueness::Unique
        );
        assert_eq!(
            value(&report(&c, &c.subject_id(&inner).unwrap(), 1)).uniqueness(),
            &HtmlIdUniqueness::Unique
        );
    }
}

#[test]
fn ordinary_document_and_fragment_attachments_use_actual_children() {
    for document in [false, true] {
        let (mut b, d) = builder();
        let root = if document {
            d.clone()
        } else {
            b.add_fragment(&d).unwrap()
        };
        let own = html(&mut b, &d, "p", Some("x"));
        let other = html(&mut b, &d, "span", Some("x"));
        children(&mut b, &own, &[]);
        children(&mut b, &other, &[]);
        children(&mut b, &root, &[own.clone(), other.clone()]);
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&own).unwrap(), 1)).uniqueness(),
            &HtmlIdUniqueness::Duplicate(c.subject_id(&other).unwrap())
        );
    }
}

#[test]
fn query_subject_applicability_does_not_restrict_competitor_namespaces() {
    let (mut b, d) = builder();
    let foreign = element(
        &mut b,
        &d,
        QualifiedName::new(Namespace::svg(), None, "g"),
        OrderedAttributes::new([Attribute::new(
            KnownHtmlAttribute::Id.qualified_name(),
            "oversized",
        )])
        .unwrap(),
    );
    let text = b.add_text(&d, "x".into()).unwrap();
    let c = b.finish().unwrap();
    for n in [d, text] {
        assert_eq!(
            report(&c, &c.subject_id(&n).unwrap(), 0).results()[0].fact(),
            &HtmlSemanticFact::IdAuthor(HtmlSemanticValue::Inapplicable)
        );
    }
    assert_eq!(
        report(&c, &c.subject_id(&foreign).unwrap(), 0).results()[0].fact(),
        &HtmlSemanticFact::IdAuthor(HtmlSemanticValue::Unsupported(
            HtmlSemanticUnsupported::ForeignNamespace
        ))
    );
}

#[test]
fn original_own_and_competitor_raw_limits_are_terminal_before_comparison() {
    for (own, other) in [("oversized", "x"), ("x", "oversized")] {
        let (c, id, _) = pair(own, other);
        let cx = context(&c);
        let mut s = session(&c, &cx, limits(), 1, HtmlSemanticCancellation::new());
        let fail = s.query(&[request(&id)]).unwrap_err();
        assert!(matches!(fail.kind(), HtmlSemanticFailureKind::Attribute(_)));
        assert_eq!(s.query(&[]).unwrap_err(), fail);
        assert_eq!(s.usage().scratch_records(), 0);
    }
    let (c, id) = single(Some("abc"));
    assert_eq!(
        value(&report(&c, &id, 3)).uniqueness(),
        &HtmlIdUniqueness::Unique
    );
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 2, HtmlSemanticCancellation::new());
    assert!(matches!(
        s.query(&[request(&id)]).unwrap_err().kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
}

#[test]
fn exact_and_one_short_work_and_actual_capacity_caps_preserve_cleanup() {
    let (c, id, _) = pair("x", "y");
    let cx = context(&c);
    let observed = report(&c, &id, 1).usage();
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
            HtmlSemanticResource::AllocatedBytes,
            observed.allocated_bytes(),
        ),
        (
            HtmlSemanticResource::NestedOutputRecords,
            observed.nested_output_records(),
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
            let result = s.query(&[request(&id)]);
            if cap == exact {
                assert_eq!(
                    value(&result.unwrap()).uniqueness(),
                    &HtmlIdUniqueness::Unique
                );
            } else {
                let fail = result.unwrap_err();
                assert!(
                    matches!(fail.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r==resource),
                    "{fail:?}"
                );
                assert_eq!(s.query(&[]).unwrap_err(), fail);
            }
            assert_eq!(s.usage().scratch_records(), 0);
        }
    }
}

#[test]
fn repeated_queries_charge_retained_records_and_late_failure_discards_batch() {
    let (c, id, _) = pair("x", "y");
    let cx = context(&c);
    let one = report(&c, &id, 1).usage();
    let mut s = session(&c, &cx, limits(), 1, HtmlSemanticCancellation::new());
    let r = s.query(&[request(&id), request(&id)]).unwrap();
    assert_eq!(r.usage().nested_output_records(), 2);
    assert!(r.usage().allocated_bytes() >= 2 * one.allocated_bytes());
    for resource in [
        HtmlSemanticResource::NestedOutputRecords,
        HtmlSemanticResource::OutputRecords,
    ] {
        let mut s = session(
            &c,
            &cx,
            capped(resource, 1),
            1,
            HtmlSemanticCancellation::new(),
        );
        let fail = s.query(&[request(&id), request(&id)]).unwrap_err();
        assert!(
            matches!(fail.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r==resource)
        );
        assert_eq!(fail.qualification().unwrap().capture(), c.identity());
        assert_eq!(fail.qualification().unwrap().context(), cx.identity());
        assert_eq!(s.query(&[]).unwrap_err(), fail);
        assert_eq!(s.usage().scratch_records(), 0);
    }
}

#[test]
fn cancellation_is_sticky_and_preserves_qualification_without_results() {
    let (c, id) = single(Some("x"));
    let cx = context(&c);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut s = session(&c, &cx, limits(), 1, cancel);
    let fail = s.query(&[request(&id)]).unwrap_err();
    assert_eq!(fail.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(s.query(&[]).unwrap_err(), fail);
    assert_eq!(s.usage().scratch_records(), 0);
    assert_eq!(fail.qualification().unwrap().capture(), c.identity());
}

#[test]
fn outputs_remain_immutable_and_capture_context_host_configuration_qualified() {
    let (a, own, _) = pair("x", "y");
    let (b, other, witness) = pair("x", "x");
    let ar = report(&a, &own, 1);
    let br = report(&b, &other, 1);
    assert_eq!(value(&ar).uniqueness(), &HtmlIdUniqueness::Unique);
    assert_eq!(
        value(&br).uniqueness(),
        &HtmlIdUniqueness::Duplicate(witness)
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
    let cr = s.query(&[request(&own)]).unwrap();
    assert_ne!(cr.context(), ar.context());
    assert_ne!(cr.qualification(), ar.qualification());
    assert_eq!(value(&ar).uniqueness(), &HtmlIdUniqueness::Unique);
}

#[test]
fn foreign_subject_and_stale_context_fail_before_successful_batch() {
    let (a, own) = single(Some("x"));
    let (b, other) = single(Some("x"));
    let cx = context(&a);
    let mut s = session(&a, &cx, limits(), 1, HtmlSemanticCancellation::new());
    let fail = s.query(&[request(&own), request(&other)]).unwrap_err();
    assert!(matches!(fail.kind(), HtmlSemanticFailureKind::Capture(_)));
    assert_eq!(fail.qualification().unwrap().capture(), a.identity());
    assert_eq!(s.query(&[]).unwrap_err(), fail);
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
fn an_unknown_gap_and_later_duplicate_cannot_rescue_an_earlier_terminal_raw_failure() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "div", None);
    let own = html(&mut b, &d, "p", Some("x"));
    let gap = html(&mut b, &d, "span", None);
    let oversized = html(&mut b, &d, "span", Some("oversized"));
    let witness = html(&mut b, &d, "span", Some("x"));
    children(&mut b, &own, &[]);
    children(&mut b, &root, &[own.clone(), gap, oversized, witness]);
    b.establish_root(&root).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 1, HtmlSemanticCancellation::new());
    let fail = s
        .query(&[request(&c.subject_id(&own).unwrap())])
        .unwrap_err();
    assert!(matches!(fail.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(s.usage().scratch_records(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), fail);
}

#[test]
fn namespaced_and_uppercase_own_slots_leave_the_actual_id_absent() {
    let (mut b, d) = builder();
    let own = element(
        &mut b,
        &d,
        KnownHtmlElement::Div.qualified_name(),
        OrderedAttributes::new([
            Attribute::new(
                QualifiedName::new(Namespace::svg(), None, "id"),
                "oversized",
            ),
            Attribute::new(
                QualifiedName::new(Namespace::none(), None, "ID"),
                "oversized",
            ),
        ])
        .unwrap(),
    );
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&own).unwrap(), 0);
    assert_eq!(value(&r).local().raw(), None);
    assert_eq!(
        value(&r).local().author(),
        HtmlAttributeAuthorState::Missing
    );
    assert_eq!(value(&r).uniqueness(), &HtmlIdUniqueness::NoId);
    assert_eq!(r.usage().peak_scratch_records(), 0);
}
