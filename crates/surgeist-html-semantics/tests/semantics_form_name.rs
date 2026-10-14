use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlCapturedShadowProperties,
    HtmlFormNameAuthorReport, HtmlFormNameUniqueness, HtmlScriptingStatus,
    HtmlSemanticCancellation, HtmlSemanticDependencies, HtmlSemanticFact, HtmlSemanticFailureKind,
    HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport,
    HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticSession, HtmlSemanticUnsupported,
    HtmlSemanticValue,
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
#[path = "common/collection_limits.rs"]
mod shared_collection_limits;
use shared_collection_limits::collection_limits as capped;
use surgeist_html::*;

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

fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20000000, 100, 1000, 0, 1000000, 100000)
}

fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::FormNameAuthor)
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
fn value(r: &HtmlSemanticReport) -> &HtmlFormNameAuthorReport {
    match r.results()[0].fact() {
        HtmlSemanticFact::FormNameAuthor(HtmlSemanticValue::Known(v)) => v,
        other => panic!("{other:?}"),
    }
}

fn pair(name: &str, other: &str) -> (HtmlCapture, HtmlCaptureSubjectId, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let m = html(&mut b, &d, "form", &[("name", name)]);
    let n = html(&mut b, &d, "form", &[("name", other)]);
    children(&mut b, &m, &[]);
    children(&mut b, &n, &[]);
    children(&mut b, &d, &[m.clone(), n.clone()]);
    let c = b.finish().unwrap();
    (
        c.clone(),
        c.subject_id(&m).unwrap(),
        c.subject_id(&n).unwrap(),
    )
}

#[test]
fn optional_absence_skips_unknown_topology_and_irrelevant_raw() {
    let (mut b, d) = builder();
    let m = html(
        &mut b,
        &d,
        "form",
        &[("id", "oversized"), ("action", "large")],
    );
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&m).unwrap(), 0);
    let v = value(&r);
    assert_eq!(v.local().observation(), HtmlAttributeObservation::Missing);
    assert_eq!(v.local().author(), HtmlAttributeAuthorState::Conforming);
    assert_eq!(
        v.local().context_author(),
        HtmlAttributeContextAuthorState::NotRequired
    );
    assert_eq!(v.uniqueness(), &HtmlFormNameUniqueness::NoSpecifiedName);
    assert_eq!(r.usage().peak_scratch_records(), 0);
    assert_eq!(r.usage().nested_output_records(), 1);
}

#[test]
fn literal_document_names_are_exact_unique_or_actual_qualified_duplicate() {
    for (a, b, duplicate) in [
        ("x", "x", true),
        ("x", "X", false),
        (" x", "x", false),
        ("x ", "x", false),
        ("é", "e\u{301}", false),
        (" ", " ", true),
    ] {
        let (c, m, n) = pair(a, b);
        let r = report(&c, &m, 10);
        let v = value(&r);
        assert_eq!(
            v.uniqueness(),
            &if duplicate {
                HtmlFormNameUniqueness::Duplicate(n)
            } else {
                HtmlFormNameUniqueness::Unique
            }
        );
        assert_eq!(v.local().author(), HtmlAttributeAuthorState::Conforming);
        assert_eq!(v.local().raw(), Some(&DocumentString::from(a)));
        assert_eq!(
            v.local().intrinsic().unwrap().value(),
            &DocumentString::from(a)
        );
        assert_eq!(
            v.local().context_author(),
            HtmlAttributeContextAuthorState::Unchecked
        );
    }
}

#[test]
fn specified_empty_compares_actual_empty_attributes_but_never_missing_reflection() {
    for other in [None, Some(""), Some("x")] {
        let (mut b, d) = builder();
        let m = html(&mut b, &d, "form", &[("name", "")]);
        let n = html(
            &mut b,
            &d,
            "form",
            &other.map_or(vec![], |v| vec![("name", v)]),
        );
        children(&mut b, &m, &[]);
        children(&mut b, &n, &[]);
        children(&mut b, &d, &[m.clone(), n.clone()]);
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&m).unwrap(), 1);
        assert_eq!(
            value(&r).local().author(),
            HtmlAttributeAuthorState::Nonconforming(
                HtmlAttributeConformanceError::FormNameMustNotBeEmpty
            )
        );
        assert_eq!(
            value(&r).uniqueness(),
            &if other == Some("") {
                HtmlFormNameUniqueness::Duplicate(c.subject_id(&n).unwrap())
            } else {
                HtmlFormNameUniqueness::Unique
            }
        );
        assert_eq!(value(&r).local().intrinsic(), None);
    }
}

#[test]
fn detached_element_and_fragment_components_have_no_document_forms_collection() {
    for fragment in [false, true] {
        let (mut b, d) = builder();
        let root = if fragment {
            b.add_fragment(&d).unwrap()
        } else {
            html(&mut b, &d, "div", &[])
        };
        let m = html(&mut b, &d, "form", &[("name", "x")]);
        let n = html(&mut b, &d, "form", &[("name", "oversized")]);
        children(&mut b, &root, &[m.clone(), n]);
        if !fragment {
            b.establish_root(&root).unwrap();
        }
        let c = b.finish().unwrap();
        // Both name-bearing forms have the Document as node_document, but
        // ordinary roots are detached; no sibling name or child coverage matters.
        assert_eq!(
            value(&report(&c, &c.subject_id(&m).unwrap(), 1)).uniqueness(),
            &HtmlFormNameUniqueness::NoFormsCollection
        );
    }
}

#[test]
fn detached_form_itself_is_not_a_synthetic_forms_collection_root() {
    let (mut b, d) = builder();
    let m = html(&mut b, &d, "form", &[("name", "x")]);
    let nested = html(&mut b, &d, "form", &[("name", "x")]);
    children(&mut b, &m, std::slice::from_ref(&nested));
    b.establish_root(&m).unwrap();
    let c = b.finish().unwrap();
    for id in [c.subject_id(&m).unwrap(), c.subject_id(&nested).unwrap()] {
        assert_eq!(
            value(&report(&c, &id, 1)).uniqueness(),
            &HtmlFormNameUniqueness::NoFormsCollection
        );
    }
}

#[test]
fn distinct_documents_and_detached_records_do_not_share_a_forms_collection() {
    let (mut b, d) = builder();
    let other_document = b
        .add_document(
            HtmlDocumentMode::Quirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let m = html(&mut b, &d, "form", &[("name", "x")]);
    let n = html(&mut b, &other_document, "form", &[("name", "x")]);
    let detached = html(&mut b, &d, "form", &[("name", "x")]);
    children(&mut b, &m, &[]);
    children(&mut b, &n, &[]);
    children(&mut b, &d, std::slice::from_ref(&m));
    children(&mut b, &other_document, std::slice::from_ref(&n));
    b.establish_root(&detached).unwrap();
    let c = b.finish().unwrap();
    for subject in [m, n] {
        assert_eq!(
            value(&report(&c, &c.subject_id(&subject).unwrap(), 1)).uniqueness(),
            &HtmlFormNameUniqueness::Unique
        );
    }
    assert_eq!(
        value(&report(&c, &c.subject_id(&detached).unwrap(), 1)).uniqueness(),
        &HtmlFormNameUniqueness::NoFormsCollection
    );
}

#[test]
fn unknown_ancestry_cannot_prove_duplicate_from_the_known_component() {
    for other in ["x", "oversized"] {
        let (mut b, d) = builder();
        let root = html(&mut b, &d, "div", &[]);
        let m = html(&mut b, &d, "form", &[("name", "x")]);
        let n = html(&mut b, &d, "form", &[("name", other)]);
        children(&mut b, &root, &[m.clone(), n]);
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&m).unwrap(), 1);
        assert_eq!(
            value(&r).uniqueness(),
            &HtmlFormNameUniqueness::Missing(HtmlSemanticMissing::Parent(
                c.subject_id(&root).unwrap()
            ))
        );
        assert_eq!(r.usage().peak_scratch_records(), 0);
    }
}

#[test]
fn document_membership_plus_later_witness_resolves_unknown_child_coverage() {
    for duplicate in [false, true] {
        let (mut b, d) = builder();
        let m = html(&mut b, &d, "form", &[("name", "x")]);
        let gap = html(&mut b, &d, "div", &[]);
        let n = html(
            &mut b,
            &d,
            "form",
            &[("name", if duplicate { "x" } else { "y" })],
        );
        children(&mut b, &m, &[]);
        children(&mut b, &n, &[]);
        children(&mut b, &d, &[m.clone(), gap.clone(), n.clone()]);
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&m).unwrap(), 1);
        assert_eq!(
            value(&r).uniqueness(),
            &if duplicate {
                HtmlFormNameUniqueness::Duplicate(c.subject_id(&n).unwrap())
            } else {
                HtmlFormNameUniqueness::Missing(HtmlSemanticMissing::OrderedChildren(
                    c.subject_id(&gap).unwrap(),
                ))
            }
        );
        assert_eq!(r.usage().scratch_records(), 0);
    }
}

#[test]
fn duplicate_skips_its_unknown_children_and_later_oversized_name() {
    let (mut b, d) = builder();
    let m = html(&mut b, &d, "form", &[("name", "x")]);
    let n = html(&mut b, &d, "form", &[("name", "x")]);
    let later = html(&mut b, &d, "form", &[("name", "oversized")]);
    children(&mut b, &m, &[]);
    children(&mut b, &d, &[m.clone(), n.clone(), later]);
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&m).unwrap(), 1)).uniqueness(),
        &HtmlFormNameUniqueness::Duplicate(c.subject_id(&n).unwrap())
    );
}

#[test]
fn only_exact_html_form_and_specified_no_namespace_lowercase_name_compete() {
    let (mut b, d) = builder();
    let m = html(&mut b, &d, "form", &[("name", "x")]);
    let mut nodes = vec![m.clone()];
    for (ns, local, attrs) in [
        (
            Namespace::svg(),
            "form",
            OrderedAttributes::new([Attribute::new(
                KnownHtmlAttribute::Name.qualified_name(),
                "oversized",
            )])
            .unwrap(),
        ),
        (
            Namespace::none(),
            "form",
            OrderedAttributes::new([Attribute::new(
                KnownHtmlAttribute::Name.qualified_name(),
                "oversized",
            )])
            .unwrap(),
        ),
        (
            Namespace::html(),
            "FORM",
            OrderedAttributes::new([Attribute::new(
                KnownHtmlAttribute::Name.qualified_name(),
                "oversized",
            )])
            .unwrap(),
        ),
        (
            Namespace::html(),
            "input",
            OrderedAttributes::new([Attribute::new(
                KnownHtmlAttribute::Name.qualified_name(),
                "oversized",
            )])
            .unwrap(),
        ),
        (
            Namespace::html(),
            "form",
            OrderedAttributes::new([Attribute::new(KnownHtmlAttribute::Id.qualified_name(), "x")])
                .unwrap(),
        ),
        (
            Namespace::html(),
            "form",
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::svg(), None, "name"),
                "oversized",
            )])
            .unwrap(),
        ),
        (
            Namespace::html(),
            "form",
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), None, "NAME"),
                "oversized",
            )])
            .unwrap(),
        ),
    ] {
        nodes.push(element(
            &mut b,
            &d,
            QualifiedName::new(ns, None, local),
            attrs,
        ));
    }
    for node in &nodes {
        children(&mut b, node, &[]);
    }
    children(&mut b, &d, &nodes);
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&m).unwrap(), 1)).uniqueness(),
        &HtmlFormNameUniqueness::Unique
    );
}

#[test]
fn foreign_wrappers_do_not_prune_html_form_descendants_and_prefixes_do_not_change_identity() {
    let (mut b, d) = builder();
    let m = html(&mut b, &d, "form", &[("name", "x")]);
    let wrapper = element(
        &mut b,
        &d,
        QualifiedName::new(Namespace::svg(), None, "g"),
        OrderedAttributes::new([]).unwrap(),
    );
    let n = element(
        &mut b,
        &d,
        QualifiedName::new(Namespace::html(), Some("h".into()), "form"),
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::none(), Some("a".into()), "name"),
            "x",
        )])
        .unwrap(),
    );
    children(&mut b, &m, &[]);
    children(&mut b, &n, &[]);
    children(&mut b, &wrapper, std::slice::from_ref(&n));
    children(&mut b, &d, &[m.clone(), wrapper]);
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&m).unwrap(), 1)).uniqueness(),
        &HtmlFormNameUniqueness::Duplicate(c.subject_id(&n).unwrap())
    );
}

#[test]
fn shadow_and_template_associations_are_excluded_from_document_forms() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let m = html(&mut b, &d, "form", &[("name", "x")]);
        let host = html(&mut b, &d, if shadow { "div" } else { "template" }, &[]);
        let f = if shadow {
            b.add_shadow_root(
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
        let n = html(&mut b, &d, "form", &[("name", "x")]);
        children(&mut b, &n, &[]);
        children(&mut b, &f, std::slice::from_ref(&n));
        children(&mut b, &host, &[]);
        children(&mut b, &m, &[]);
        children(&mut b, &d, &[m.clone(), host]);
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&m).unwrap(), 1)).uniqueness(),
            &HtmlFormNameUniqueness::Unique
        );
        assert_eq!(
            value(&report(&c, &c.subject_id(&n).unwrap(), 1)).uniqueness(),
            &HtmlFormNameUniqueness::NoFormsCollection
        );
    }
}

#[test]
fn surrogate_null_names_compare_as_exact_units_without_unicode_replacement() {
    for duplicate in [false, true] {
        let (mut b, d) = builder();
        let name = DocumentString::from_utf16(vec![0xD800, 0, 0xDC00]);
        let m = element(
            &mut b,
            &d,
            KnownHtmlElement::Form.qualified_name(),
            OrderedAttributes::new([Attribute::new(
                KnownHtmlAttribute::Name.qualified_name(),
                name.clone(),
            )])
            .unwrap(),
        );
        let n = element(
            &mut b,
            &d,
            KnownHtmlElement::Form.qualified_name(),
            OrderedAttributes::new([Attribute::new(
                KnownHtmlAttribute::Name.qualified_name(),
                DocumentString::from_utf16(vec![
                    0xD800,
                    0,
                    if duplicate { 0xDC00 } else { 0xFFFD },
                ]),
            )])
            .unwrap(),
        );
        children(&mut b, &m, &[]);
        children(&mut b, &n, &[]);
        children(&mut b, &d, &[m.clone(), n.clone()]);
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&m).unwrap(), 3);
        assert_eq!(value(&r).local().raw(), Some(&name));
        assert_eq!(value(&r).local().intrinsic().unwrap().value(), &name);
        assert_eq!(
            value(&r).uniqueness(),
            &if duplicate {
                HtmlFormNameUniqueness::Duplicate(c.subject_id(&n).unwrap())
            } else {
                HtmlFormNameUniqueness::Unique
            }
        );
    }
}

#[test]
fn wrong_subject_roles_do_not_consult_irrelevant_raw_and_foreign_is_unsupported() {
    let (mut b, d) = builder();
    let wrong = html(&mut b, &d, "FORM", &[("name", "huge")]);
    let foreign = element(
        &mut b,
        &d,
        QualifiedName::new(Namespace::svg(), None, "form"),
        OrderedAttributes::new([Attribute::new(
            KnownHtmlAttribute::Name.qualified_name(),
            "huge",
        )])
        .unwrap(),
    );
    let t = b.add_text(&d, "x".into()).unwrap();
    let c = b.finish().unwrap();
    for node in [wrong, t, d] {
        assert_eq!(
            report(&c, &c.subject_id(&node).unwrap(), 0).results()[0].fact(),
            &HtmlSemanticFact::FormNameAuthor(HtmlSemanticValue::Inapplicable)
        );
    }
    assert_eq!(
        report(&c, &c.subject_id(&foreign).unwrap(), 0).results()[0].fact(),
        &HtmlSemanticFact::FormNameAuthor(HtmlSemanticValue::Unsupported(
            HtmlSemanticUnsupported::ForeignNamespace
        ))
    );
}

#[test]
fn whole_own_and_competing_raw_caps_are_terminal_before_comparison() {
    for (a, b) in [("oversized", "x"), ("x", "oversized")] {
        let (c, m, _) = pair(a, b);
        let cx = context(&c);
        let mut s = session(&c, &cx, limits(), 1, HtmlSemanticCancellation::new());
        let failure = s.query(&[request(&m)]).unwrap_err();
        assert!(matches!(
            failure.kind(),
            HtmlSemanticFailureKind::Attribute(_)
        ));
        assert_eq!(s.query(&[]).unwrap_err(), failure);
        assert_eq!(s.usage().scratch_records(), 0);
    }
    let (c, m, _) = pair("abc", "def");
    assert_eq!(
        value(&report(&c, &m, 3)).uniqueness(),
        &HtmlFormNameUniqueness::Unique
    );
    let cx = context(&c);
    assert!(
        session(&c, &cx, limits(), 2, HtmlSemanticCancellation::new())
            .query(&[request(&m)])
            .is_err()
    );
}

#[test]
fn terminal_raw_failure_after_unknown_gap_cannot_be_rescued_by_later_duplicate() {
    let (mut b, d) = builder();
    let m = html(&mut b, &d, "form", &[("name", "x")]);
    let gap = html(&mut b, &d, "div", &[]);
    let huge = html(&mut b, &d, "form", &[("name", "oversized")]);
    let n = html(&mut b, &d, "form", &[("name", "x")]);
    children(&mut b, &m, &[]);
    children(&mut b, &d, &[m.clone(), gap, huge, n]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 1, HtmlSemanticCancellation::new());
    let failure = s.query(&[request(&c.subject_id(&m).unwrap())]).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    assert_eq!(s.query(&[]).unwrap_err(), failure);
    assert_eq!(s.usage().scratch_records(), 0);
}

#[test]
fn exact_and_one_short_caps_keep_actual_storage_atomic_and_cleanup_scratch() {
    let (c, m, _) = pair("x", "y");
    let cx = context(&c);
    let usage = report(&c, &m, 1).usage();
    assert_eq!(usage.scratch_records(), 0);
    assert_eq!(usage.nested_output_records(), 1);
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
        (
            HtmlSemanticResource::NestedOutputRecords,
            usage.nested_output_records(),
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
            let r = s.query(&[request(&m)]);
            if cap == exact {
                assert_eq!(
                    value(&r.unwrap()).uniqueness(),
                    &HtmlFormNameUniqueness::Unique
                );
            } else {
                let failure = r.unwrap_err();
                assert!(
                    matches!(failure.kind(), HtmlSemanticFailureKind::LimitExceeded { resource: r, .. } if *r == resource),
                    "{failure:?}"
                );
                assert_eq!(s.query(&[]).unwrap_err(), failure);
            }
            assert_eq!(s.usage().scratch_records(), 0);
        }
    }
}

#[test]
fn repeated_results_retain_distinct_records_and_late_failure_discards_batch() {
    let (c, m, _) = pair("x", "y");
    let cx = context(&c);
    let one = report(&c, &m, 1).usage();
    let mut s = session(&c, &cx, limits(), 1, HtmlSemanticCancellation::new());
    let r = s.query(&[request(&m), request(&m)]).unwrap();
    assert_eq!(
        r.usage().nested_output_records(),
        2 * one.nested_output_records()
    );
    assert!(r.usage().allocated_bytes() >= 2 * one.allocated_bytes());
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
            matches!(failure.kind(), HtmlSemanticFailureKind::LimitExceeded {resource: r,..} if *r == resource)
        );
        assert_eq!(failure.qualification().unwrap().capture(), c.identity());
        assert_eq!(failure.qualification().unwrap().context(), cx.identity());
        assert_eq!(s.query(&[]).unwrap_err(), failure);
        assert_eq!(s.usage().scratch_records(), 0);
    }
}

#[test]
fn cancellation_is_first_sticky_failure_even_after_a_delivered_report() {
    let (c, m, _) = pair("x", "y");
    let cx = context(&c);
    let cancel = HtmlSemanticCancellation::new();
    let mut s = session(&c, &cx, limits(), 1, cancel.clone());
    let old = s.query(&[request(&m)]).unwrap();
    cancel.cancel();
    let failure = s.query(&[request(&m)]).unwrap_err();
    assert_eq!(failure.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(s.query(&[]).unwrap_err(), failure);
    assert_eq!(failure.qualification().unwrap().capture(), c.identity());
    assert_eq!(value(&old).uniqueness(), &HtmlFormNameUniqueness::Unique);
    assert_eq!(s.usage().scratch_records(), 0);
}

#[test]
fn changed_capture_and_context_keep_old_reports_immutable_and_qualified() {
    let (a, m, _) = pair("x", "y");
    let (b, n, duplicate) = pair("x", "x");
    let ar = report(&a, &m, 1);
    let br = report(&b, &n, 1);
    assert_eq!(value(&ar).uniqueness(), &HtmlFormNameUniqueness::Unique);
    assert_eq!(
        value(&br).uniqueness(),
        &HtmlFormNameUniqueness::Duplicate(duplicate)
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
    assert_ne!(ar.qualification(), cr.qualification());
    assert_eq!(value(&ar).uniqueness(), &HtmlFormNameUniqueness::Unique);
}

#[test]
fn foreign_subject_late_batch_and_stale_context_are_terminal_rejections() {
    let (a, m, _) = pair("x", "y");
    let (b, n, _) = pair("x", "x");
    let cx = context(&a);
    let mut s = session(&a, &cx, limits(), 1, HtmlSemanticCancellation::new());
    let failure = s.query(&[request(&m), request(&n)]).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Capture(_)
    ));
    assert_eq!(failure.qualification().unwrap().capture(), a.identity());
    assert_eq!(s.query(&[]).unwrap_err(), failure);
    assert_eq!(s.usage().scratch_records(), 0);
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
fn namespaced_and_uppercase_own_names_do_not_become_supplied_name() {
    let (mut b, d) = builder();
    let m = element(
        &mut b,
        &d,
        KnownHtmlElement::Form.qualified_name(),
        OrderedAttributes::new([
            Attribute::new(
                QualifiedName::new(Namespace::svg(), None, "name"),
                "oversized",
            ),
            Attribute::new(
                QualifiedName::new(Namespace::none(), None, "NAME"),
                "oversized",
            ),
        ])
        .unwrap(),
    );
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&m).unwrap(), 0);
    assert_eq!(
        value(&r).uniqueness(),
        &HtmlFormNameUniqueness::NoSpecifiedName
    );
    assert_eq!(value(&r).local().raw(), None);
}
