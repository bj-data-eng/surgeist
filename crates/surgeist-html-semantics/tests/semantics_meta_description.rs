use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlCapturedShadowProperties,
    HtmlMetaDescriptionAuthorReport, HtmlMetaDescriptionContentRequirement,
    HtmlMetaDescriptionUniqueness, HtmlScriptingStatus, HtmlSemanticCancellation, HtmlSemanticFact,
    HtmlSemanticFailureKind, HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery,
    HtmlSemanticReport, HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticUnsupported,
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

// HTML 877d #meta-description, #attr-meta-content and #attr-meta-name:
// ASCII-insensitive description names, required content independent of effective
// empty metadata, and at most one description per actual Document.

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
fn attrs(name: Option<&str>, content: Option<DocumentString>) -> OrderedAttributes {
    let mut a = Vec::new();
    if let Some(n) = name {
        a.push(Attribute::new(KnownHtmlAttribute::Name.qualified_name(), n));
    }
    if let Some(c) = content {
        a.push(Attribute::new(
            KnownHtmlAttribute::Content.qualified_name(),
            c,
        ));
    }
    OrderedAttributes::new(a).unwrap()
}
fn meta(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    name: Option<&str>,
    content: Option<&str>,
) -> HtmlCaptureBuilderSubject {
    element(
        b,
        d,
        KnownHtmlElement::Meta.qualified_name(),
        attrs(name, content.map(DocumentString::from)),
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
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::MetaDescriptionAuthor)
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
fn value(r: &HtmlSemanticReport) -> &HtmlMetaDescriptionAuthorReport {
    match r.results()[0].fact() {
        HtmlSemanticFact::MetaDescriptionAuthor(HtmlSemanticValue::Known(v)) => v,
        other => panic!("{other:?}"),
    }
}
fn single(name: Option<&str>, content: Option<&str>) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let n = meta(&mut b, &d, name, content);
    children(&mut b, &n, &[]);
    children(&mut b, &d, std::slice::from_ref(&n));
    let c = b.finish().unwrap();
    let id = c.subject_id(&n).unwrap();
    (c, id)
}
fn pair(first: &str, second: &str) -> (HtmlCapture, HtmlCaptureSubjectId, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let n = meta(&mut b, &d, Some(first), Some("a"));
    let z = meta(&mut b, &d, Some(second), None);
    children(&mut b, &n, &[]);
    children(&mut b, &z, &[]);
    children(&mut b, &d, &[n.clone(), z.clone()]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&n).unwrap();
    let zi = c.subject_id(&z).unwrap();
    (c, id, zi)
}
#[test]
fn unique_description_retains_exact_pair_and_unchecked_appropriateness() {
    let (c, id) = single(Some("DeScRiPtIoN"), Some("  a directory description\r\n"));
    let r = report(&c, &id, 100);
    let v = value(&r);
    assert_eq!(
        v.name_attribute().value().code_units(),
        DocumentString::from("DeScRiPtIoN").code_units()
    );
    assert_eq!(
        v.effective_content().code_units(),
        DocumentString::from("  a directory description\r\n").code_units()
    );
    assert_eq!(
        v.content_requirement(),
        HtmlMetaDescriptionContentRequirement::Satisfied
    );
    assert_eq!(
        v.content_appropriateness(),
        HtmlAttributeContextAuthorState::Unchecked
    );
    assert_eq!(v.uniqueness(), &HtmlMetaDescriptionUniqueness::Unique);
    assert_eq!(r.qualification().capture(), c.identity());
}
#[test]
fn absent_and_empty_content_share_effective_empty_but_have_distinct_author_presence() {
    for content in [None, Some("")] {
        let (c, id) = single(Some("description"), content);
        let r = report(&c, &id, 11);
        let v = value(&r);
        assert!(v.effective_content().is_empty());
        assert_eq!(v.content_attribute().is_some(), content.is_some());
        assert_eq!(
            v.content_requirement(),
            if content.is_some() {
                HtmlMetaDescriptionContentRequirement::Satisfied
            } else {
                HtmlMetaDescriptionContentRequirement::MissingContent
            }
        );
        assert_eq!(v.uniqueness(), &HtmlMetaDescriptionUniqueness::Unique);
    }
}
#[test]
fn mixed_case_competitor_without_content_still_proves_duplicate() {
    let (c, id, z) = pair("description", "DESCRIPTION");
    assert_eq!(
        value(&report(&c, &id, 11)).uniqueness(),
        &HtmlMetaDescriptionUniqueness::Duplicate(z.clone())
    );
    let r = report(&c, &z, 11);
    let v = value(&r);
    assert_eq!(
        v.content_requirement(),
        HtmlMetaDescriptionContentRequirement::MissingContent
    );
    assert_eq!(
        v.uniqueness(),
        &HtmlMetaDescriptionUniqueness::Duplicate(id)
    );
}
#[test]
fn absence_other_names_padding_and_non_ascii_case_never_select_description() {
    for name in [
        None,
        Some("keywords"),
        Some(" description"),
        Some("description "),
        Some("deſcription"),
        Some("description\u{a0}"),
    ] {
        let (mut b, d) = builder();
        let n = meta(&mut b, &d, name, None);
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&n).unwrap(), 100);
        assert!(matches!(
            r.results()[0].fact(),
            HtmlSemanticFact::MetaDescriptionAuthor(HtmlSemanticValue::Inapplicable)
        ));
    }
}
#[test]
fn namespaced_or_case_changed_attribute_names_do_not_select_description() {
    for name in [
        QualifiedName::new(Namespace::from_uri("urn:attr"), Some("p".into()), "name"),
        QualifiedName::new(Namespace::none(), None, "NAME"),
    ] {
        let (mut b, d) = builder();
        let n = element(
            &mut b,
            &d,
            KnownHtmlElement::Meta.qualified_name(),
            OrderedAttributes::new([Attribute::new(name, "description")]).unwrap(),
        );
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&n).unwrap(), 11);
        assert!(matches!(
            r.results()[0].fact(),
            HtmlSemanticFact::MetaDescriptionAuthor(HtmlSemanticValue::Inapplicable)
        ));
    }
}
#[test]
fn prefix_does_not_replace_namespace_or_exact_local_name() {
    for (ns, local, foreign) in [
        (Namespace::html(), "meta", false),
        (Namespace::html(), "META", false),
        (Namespace::svg(), "meta", true),
    ] {
        let (mut b, d) = builder();
        let n = element(
            &mut b,
            &d,
            QualifiedName::new(ns, Some("x".into()), local),
            attrs(Some("description"), Some("x".into())),
        );
        children(&mut b, &n, &[]);
        children(&mut b, &d, std::slice::from_ref(&n));
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&n).unwrap(), 11);
        match r.results()[0].fact() {
            HtmlSemanticFact::MetaDescriptionAuthor(HtmlSemanticValue::Known(_)) => {
                assert_eq!(local, "meta")
            }
            HtmlSemanticFact::MetaDescriptionAuthor(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            )) => assert!(foreign),
            HtmlSemanticFact::MetaDescriptionAuthor(HtmlSemanticValue::Inapplicable) => {
                assert_eq!(local, "META")
            }
            x => panic!("{x:?}"),
        }
    }
}
#[test]
fn description_in_body_counts_without_head_only_approximation() {
    let (mut b, d) = builder();
    let body = element(
        &mut b,
        &d,
        KnownHtmlElement::Body.qualified_name(),
        OrderedAttributes::new([]).unwrap(),
    );
    let a = meta(&mut b, &d, Some("description"), Some("head-like"));
    let z = meta(&mut b, &d, Some("description"), Some("body"));
    children(&mut b, &a, &[]);
    children(&mut b, &z, &[]);
    children(&mut b, &body, std::slice::from_ref(&z));
    children(&mut b, &d, &[a.clone(), body]);
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&a).unwrap(), 20)).uniqueness(),
        &HtmlMetaDescriptionUniqueness::Duplicate(c.subject_id(&z).unwrap())
    );
}
#[test]
fn foreign_meta_and_other_metadata_names_are_not_competitors() {
    let (mut b, d) = builder();
    let a = meta(&mut b, &d, Some("description"), None);
    let z = element(
        &mut b,
        &d,
        QualifiedName::new(Namespace::svg(), None, "meta"),
        attrs(Some("description"), None),
    );
    let k = meta(&mut b, &d, Some("keywords"), None);
    for n in [&a, &z, &k] {
        children(&mut b, n, &[]);
    }
    children(&mut b, &d, &[a.clone(), z, k]);
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&a).unwrap(), 11)).uniqueness(),
        &HtmlMetaDescriptionUniqueness::Unique
    );
}
#[test]
fn detached_element_or_fragment_has_no_document_scope_even_with_equal_descriptions() {
    for fragment in [false, true] {
        let (mut b, d) = builder();
        let root = if fragment {
            b.add_fragment(&d).unwrap()
        } else {
            element(
                &mut b,
                &d,
                KnownHtmlElement::Div.qualified_name(),
                OrderedAttributes::new([]).unwrap(),
            )
        };
        let a = meta(&mut b, &d, Some("description"), None);
        let z = meta(&mut b, &d, Some("description"), Some("x"));
        children(&mut b, &a, &[]);
        children(&mut b, &z, &[]);
        children(&mut b, &root, &[a.clone(), z]);
        if !fragment {
            b.establish_root(&root).unwrap();
        }
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&a).unwrap(), 11)).uniqueness(),
            &HtmlMetaDescriptionUniqueness::NoDocumentMetadataScope
        );
    }
}
#[test]
fn unknown_ancestry_cannot_be_resolved_by_an_equal_meta_in_the_known_component() {
    let (mut b, d) = builder();
    let p = element(
        &mut b,
        &d,
        KnownHtmlElement::Div.qualified_name(),
        OrderedAttributes::new([]).unwrap(),
    );
    let a = meta(&mut b, &d, Some("description"), None);
    let z = meta(&mut b, &d, Some("description"), None);
    children(&mut b, &a, &[]);
    children(&mut b, &z, &[]);
    children(&mut b, &p, &[a.clone(), z]);
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&a).unwrap(), 11)).uniqueness(),
        &HtmlMetaDescriptionUniqueness::Missing(HtmlSemanticMissing::Parent(
            c.subject_id(&p).unwrap()
        ))
    );
}
#[test]
fn document_duplicate_witness_resolves_unrelated_unknown_children_but_absence_does_not() {
    for duplicate in [false, true] {
        let (mut b, d) = builder();
        let a = meta(&mut b, &d, Some("description"), None);
        let gap = element(
            &mut b,
            &d,
            KnownHtmlElement::Div.qualified_name(),
            OrderedAttributes::new([]).unwrap(),
        );
        let z = meta(
            &mut b,
            &d,
            Some(if duplicate { "description" } else { "keywords" }),
            None,
        );
        children(&mut b, &a, &[]);
        children(&mut b, &z, &[]);
        children(&mut b, &d, &[a.clone(), gap.clone(), z.clone()]);
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&a).unwrap(), 11)).uniqueness(),
            &if duplicate {
                HtmlMetaDescriptionUniqueness::Duplicate(c.subject_id(&z).unwrap())
            } else {
                HtmlMetaDescriptionUniqueness::Missing(HtmlSemanticMissing::OrderedChildren(
                    c.subject_id(&gap).unwrap(),
                ))
            }
        );
    }
}
#[test]
fn template_and_shadow_contents_do_not_join_host_document_metadata() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let a = meta(&mut b, &d, Some("description"), None);
        let host = element(
            &mut b,
            &d,
            if shadow {
                KnownHtmlElement::Div.qualified_name()
            } else {
                KnownHtmlElement::Template.qualified_name()
            },
            OrderedAttributes::new([]).unwrap(),
        );
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
        let inner = meta(&mut b, &d, Some("description"), None);
        children(&mut b, &inner, &[]);
        children(&mut b, &f, std::slice::from_ref(&inner));
        children(&mut b, &host, &[]);
        children(&mut b, &a, &[]);
        children(&mut b, &d, &[a.clone(), host]);
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&a).unwrap(), 11)).uniqueness(),
            &HtmlMetaDescriptionUniqueness::Unique
        );
        assert_eq!(
            value(&report(&c, &c.subject_id(&inner).unwrap(), 11)).uniqueness(),
            &HtmlMetaDescriptionUniqueness::NoDocumentMetadataScope
        );
    }
}
#[test]
fn effective_content_preserves_null_unpaired_surrogates_and_non_ascii_whitespace() {
    let (mut b, d) = builder();
    let raw = DocumentString::from_utf16(vec![0xd800, 0, 0xdc00, 0xa0, 10, 32]);
    let a = element(
        &mut b,
        &d,
        KnownHtmlElement::Meta.qualified_name(),
        attrs(Some("description"), Some(raw.clone())),
    );
    children(&mut b, &a, &[]);
    children(&mut b, &d, std::slice::from_ref(&a));
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&a).unwrap(), 11);
    let v = value(&r);
    assert_eq!(v.effective_content().code_units(), raw.code_units());
    assert_eq!(
        v.content_attribute().unwrap().value().code_units(),
        raw.code_units()
    );
    assert_eq!(
        v.content_requirement(),
        HtmlMetaDescriptionContentRequirement::Satisfied
    );
}
#[test]
fn missing_content_local_failure_does_not_erase_missing_context() {
    let (mut b, d) = builder();
    let a = meta(&mut b, &d, Some("description"), None);
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    let r = report(&c, &id, 11);
    let v = value(&r);
    assert_eq!(
        v.content_requirement(),
        HtmlMetaDescriptionContentRequirement::MissingContent
    );
    assert_eq!(
        v.uniqueness(),
        &HtmlMetaDescriptionUniqueness::Missing(HtmlSemanticMissing::Parent(id))
    );
}
#[test]
fn raw_caps_cover_own_name_content_and_competitor_name_before_processing() {
    let (c, id) = single(Some("description"), None);
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 10, HtmlSemanticCancellation::new());
    assert!(matches!(
        s.query(&[request(&id)]).unwrap_err().kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    let (c, id) = single(Some("description"), Some("123456789012"));
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 11, HtmlSemanticCancellation::new());
    assert!(matches!(
        s.query(&[request(&id)]).unwrap_err().kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    let (c, id, _) = pair("description", "123456789012");
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 11, HtmlSemanticCancellation::new());
    let fail = s.query(&[request(&id)]).unwrap_err();
    assert!(matches!(fail.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(s.query(&[]).unwrap_err(), fail);
    assert_eq!(s.usage().scratch_records(), 0);
}
#[test]
fn competitor_content_is_irrelevant_to_duplicate_proof_even_when_over_raw_cap() {
    let (mut b, d) = builder();
    let a = meta(&mut b, &d, Some("description"), None);
    let z = meta(
        &mut b,
        &d,
        Some("DESCRIPTION"),
        Some("12345678901234567890"),
    );
    children(&mut b, &a, &[]);
    children(&mut b, &z, &[]);
    children(&mut b, &d, &[a.clone(), z.clone()]);
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&a).unwrap(), 11)).uniqueness(),
        &HtmlMetaDescriptionUniqueness::Duplicate(c.subject_id(&z).unwrap())
    );
}

#[test]
fn actual_work_and_capacities_accept_exact_and_reject_one_short_with_balanced_scratch() {
    let (c, id) = single(Some("description"), Some("x"));
    let cx = context(&c);
    let usage = report(&c, &id, 11).usage();
    assert_eq!(usage.nested_output_records(), 1);
    assert_eq!(usage.scratch_records(), 0);
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
                11,
                HtmlSemanticCancellation::new(),
            );
            let r = s.query(&[request(&id)]);
            if cap == exact {
                assert_eq!(
                    value(&r.unwrap()).uniqueness(),
                    &HtmlMetaDescriptionUniqueness::Unique
                );
            } else {
                let fail = r.unwrap_err();
                assert!(
                    matches!(fail.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..}if *r==resource),
                    "{fail:?}"
                );
                assert_eq!(s.query(&[]).unwrap_err(), fail);
            }
            assert_eq!(s.usage().scratch_records(), 0);
        }
    }
}
#[test]
fn cancellation_is_sticky_and_preserves_exact_qualification() {
    let (c, id) = single(Some("description"), None);
    let cx = context(&c);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut s = session(&c, &cx, limits(), 11, cancel);
    let fail = s.query(&[request(&id)]).unwrap_err();
    assert_eq!(fail.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(fail.qualification().unwrap().capture(), c.identity());
    assert_eq!(fail.qualification().unwrap().context(), cx.identity());
    assert_eq!(s.query(&[]).unwrap_err(), fail);
    assert_eq!(s.usage().scratch_records(), 0);
}
#[test]
fn late_terminal_raw_failure_discards_all_requested_results() {
    let (mut b, d) = builder();
    let a = meta(&mut b, &d, Some("description"), None);
    let z = meta(&mut b, &d, Some("description"), Some("123456789012"));
    children(&mut b, &a, &[]);
    children(&mut b, &z, &[]);
    children(&mut b, &d, &[a.clone(), z.clone()]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 11, HtmlSemanticCancellation::new());
    let fail = s
        .query(&[
            request(&c.subject_id(&a).unwrap()),
            request(&c.subject_id(&z).unwrap()),
        ])
        .unwrap_err();
    assert!(matches!(fail.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(s.query(&[]).unwrap_err(), fail);
    assert_eq!(s.usage().scratch_records(), 0);
}
#[test]
fn repeated_results_charge_real_nested_storage_and_late_capacity_failure_is_atomic() {
    let (c, id) = single(Some("description"), None);
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 11, HtmlSemanticCancellation::new());
    let r = s.query(&[request(&id), request(&id)]).unwrap();
    assert_eq!(r.usage().nested_output_records(), 2);
    for resource in [
        HtmlSemanticResource::OutputRecords,
        HtmlSemanticResource::NestedOutputRecords,
    ] {
        let mut s = session(
            &c,
            &cx,
            capped(resource, 1),
            11,
            HtmlSemanticCancellation::new(),
        );
        let fail = s.query(&[request(&id), request(&id)]).unwrap_err();
        assert!(
            matches!(fail.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..}if *r==resource)
        );
        assert_eq!(s.query(&[]).unwrap_err(), fail);
        assert_eq!(s.usage().scratch_records(), 0);
    }
}
#[test]
fn non_element_queries_are_inapplicable_and_foreign_requests_remain_typed_failures() {
    let (c, _) = single(Some("description"), None);
    let (_other, other_id) = single(Some("description"), None);
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 11, HtmlSemanticCancellation::new());
    assert!(s.query(&[request(&other_id)]).is_err());
    let (mut b, d) = builder();
    let t = b.add_text(&d, "x".into()).unwrap();
    children(&mut b, &d, std::slice::from_ref(&t));
    let c = b.finish().unwrap();
    for subject in [&d, &t] {
        let r = report(&c, &c.subject_id(subject).unwrap(), 0);
        assert!(matches!(
            r.results()[0].fact(),
            HtmlSemanticFact::MetaDescriptionAuthor(HtmlSemanticValue::Inapplicable)
        ));
    }
}

#[test]
fn descriptions_in_other_documents_of_one_capture_do_not_compete() {
    let (mut b, d) = builder();
    let other_document = b
        .add_document(
            HtmlDocumentMode::Quirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let a = meta(&mut b, &d, Some("description"), None);
    let z = meta(&mut b, &other_document, Some("description"), None);
    children(&mut b, &a, &[]);
    children(&mut b, &z, &[]);
    children(&mut b, &d, std::slice::from_ref(&a));
    children(&mut b, &other_document, std::slice::from_ref(&z));
    let c = b.finish().unwrap();
    for subject in [&a, &z] {
        assert_eq!(
            value(&report(&c, &c.subject_id(subject).unwrap(), 11)).uniqueness(),
            &HtmlMetaDescriptionUniqueness::Unique
        );
    }
}

#[test]
fn namespaced_content_remains_absent_and_unknown_unrelated_content_is_not_admitted() {
    let (mut b, d) = builder();
    let a = element(
        &mut b,
        &d,
        KnownHtmlElement::Meta.qualified_name(),
        OrderedAttributes::new([
            Attribute::new(KnownHtmlAttribute::Name.qualified_name(), "description"),
            Attribute::new(
                QualifiedName::new(Namespace::from_uri("urn:foreign"), None, "content"),
                "12345678901234567890",
            ),
        ])
        .unwrap(),
    );
    children(&mut b, &a, &[]);
    children(&mut b, &d, std::slice::from_ref(&a));
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&a).unwrap(), 11);
    assert_eq!(
        value(&r).content_requirement(),
        HtmlMetaDescriptionContentRequirement::MissingContent
    );
    assert!(value(&r).effective_content().is_empty());
    let (c, id) = single(Some("keywords"), Some("12345678901234567890"));
    let r = report(&c, &id, 11);
    assert!(matches!(
        r.results()[0].fact(),
        HtmlSemanticFact::MetaDescriptionAuthor(HtmlSemanticValue::Inapplicable)
    ));
}

#[test]
fn a_later_duplicate_cannot_rescue_an_earlier_terminal_name_cap() {
    let (mut b, d) = builder();
    let a = meta(&mut b, &d, Some("description"), None);
    let gap = element(
        &mut b,
        &d,
        KnownHtmlElement::Div.qualified_name(),
        OrderedAttributes::new([]).unwrap(),
    );
    let over = meta(&mut b, &d, Some("123456789012"), None);
    let z = meta(&mut b, &d, Some("description"), None);
    for n in [&a, &over, &z] {
        children(&mut b, n, &[]);
    }
    children(&mut b, &d, &[a.clone(), gap, over, z]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 11, HtmlSemanticCancellation::new());
    let fail = s.query(&[request(&c.subject_id(&a).unwrap())]).unwrap_err();
    assert!(matches!(fail.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(s.query(&[]).unwrap_err(), fail);
    assert_eq!(s.usage().scratch_records(), 0);
}
