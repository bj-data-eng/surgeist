use crate::semantics::support::builder;
use crate::semantics::support::context;
use crate::semantics::support::scalar_limits as resource_limits;
use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureError,
    HtmlCaptureRegistryInput, HtmlCaptureSubjectId, HtmlCapturedIsValue,
    HtmlCapturedShadowProperties, HtmlFooterScope, HtmlScriptingStatus, HtmlSemanticCancellation,
    HtmlSemanticContext, HtmlSemanticDependencies, HtmlSemanticFact, HtmlSemanticFailureKind,
    HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport,
    HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticResult, HtmlSemanticSession,
    HtmlSemanticUnsupported, HtmlSemanticValue,
};

fn expanded(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    name: QualifiedName,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        name,
        OrderedAttributes::new([]).unwrap(),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn html(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    name: &str,
) -> HtmlCaptureBuilderSubject {
    expanded(b, d, QualifiedName::new(Namespace::html(), None, name))
}
fn children(
    b: &mut HtmlCaptureBuilder,
    parent: &HtmlCaptureBuilderSubject,
    list: &[HtmlCaptureBuilderSubject],
) {
    b.establish_children(parent, list).unwrap();
}

fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20000000, 100, 0, 0, 1000000, 0)
}
fn session<'a>(
    c: &'a HtmlCapture,
    cx: &'a HtmlSemanticContext,
    limits: HtmlSemanticLimits,
    raw: usize,
    cancel: HtmlSemanticCancellation,
) -> HtmlSemanticSession<'a> {
    HtmlSemanticSession::new(c, cx, HtmlValueInterpreter::new(raw), limits, cancel).unwrap()
}
fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::FooterScope)
}
fn report(c: &HtmlCapture, id: &HtmlCaptureSubjectId) -> HtmlSemanticReport {
    session(c, &context(c), limits(), 0, HtmlSemanticCancellation::new())
        .query(&[request(id)])
        .unwrap()
}
fn value(r: &HtmlSemanticReport) -> &HtmlSemanticValue<HtmlFooterScope> {
    match r.results()[0].fact() {
        HtmlSemanticFact::FooterScope(v) => v,
        other => panic!("expected footer scope, got {other:?}"),
    }
}
fn body_scope(
    c: &HtmlCapture,
    d: &HtmlCaptureBuilderSubject,
    body: Option<&HtmlCaptureBuilderSubject>,
) -> HtmlSemanticValue<HtmlFooterScope> {
    HtmlSemanticValue::Known(HtmlFooterScope::DocumentBody {
        document: c.document_id(&c.subject_id(d).unwrap()).unwrap(),
        body: body.map(|body| c.subject_id(body).unwrap()),
    })
}
fn section_scope(
    c: &HtmlCapture,
    section: &HtmlCaptureBuilderSubject,
) -> HtmlSemanticValue<HtmlFooterScope> {
    HtmlSemanticValue::Known(HtmlFooterScope::Section(c.subject_id(section).unwrap()))
}
fn simple() -> (HtmlCapture, HtmlCaptureSubjectId, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let section = html(&mut b, &d, "section");
    let footer = html(&mut b, &d, "footer");
    children(&mut b, &section, std::slice::from_ref(&footer));
    let c = b.finish().unwrap();
    let id = c.subject_id(&footer).unwrap();
    let owner = c.subject_id(&section).unwrap();
    (c, id, owner)
}

#[test]
fn each_of_the_four_sectioning_content_elements_supplies_scope() {
    for name in ["article", "aside", "nav", "section"] {
        let (mut b, d) = builder();
        let section = html(&mut b, &d, name);
        let footer = html(&mut b, &d, "footer");
        children(&mut b, &section, std::slice::from_ref(&footer));
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&footer).unwrap())),
            &section_scope(&c, &section)
        );
    }
}

#[test]
fn closest_section_wins_without_requiring_its_parent_or_document_children() {
    let (mut b, d) = builder();
    let outer = html(&mut b, &d, "article");
    let inner = html(&mut b, &d, "nav");
    let wrapper = html(&mut b, &d, "div");
    let footer = html(&mut b, &d, "footer");
    children(&mut b, &outer, std::slice::from_ref(&inner));
    children(&mut b, &inner, std::slice::from_ref(&wrapper));
    children(&mut b, &wrapper, std::slice::from_ref(&footer));
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&footer).unwrap())),
        &section_scope(&c, &inner)
    );
}

#[test]
fn foreign_wrapper_bridges_ordinary_ancestry_without_becoming_a_section() {
    let (mut b, d) = builder();
    let section = html(&mut b, &d, "aside");
    let foreign = expanded(
        &mut b,
        &d,
        QualifiedName::new(Namespace::svg(), None, "article"),
    );
    let footer = html(&mut b, &d, "footer");
    children(&mut b, &section, std::slice::from_ref(&foreign));
    children(&mut b, &foreign, std::slice::from_ref(&footer));
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&footer).unwrap())),
        &section_scope(&c, &section)
    );
}

#[test]
fn historical_sectioning_roots_and_nested_header_footer_are_not_sectioning_content() {
    for name in [
        "body",
        "blockquote",
        "details",
        "dialog",
        "fieldset",
        "td",
        "header",
        "footer",
    ] {
        let (mut b, d) = builder();
        let section = html(&mut b, &d, "section");
        let wrapper = html(&mut b, &d, name);
        let footer = html(&mut b, &d, "footer");
        children(&mut b, &section, std::slice::from_ref(&wrapper));
        children(&mut b, &wrapper, std::slice::from_ref(&footer));
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&footer).unwrap())),
            &section_scope(&c, &section)
        );
    }
}

#[test]
fn expanded_names_are_case_exact_and_prefix_independent_in_both_roles() {
    let (mut b, d) = builder();
    let section = expanded(
        &mut b,
        &d,
        QualifiedName::new(Namespace::html(), Some("s".into()), "section"),
    );
    let uppercase = html(&mut b, &d, "ARTICLE");
    let no_ns = expanded(
        &mut b,
        &d,
        QualifiedName::new(Namespace::none(), None, "nav"),
    );
    let malformed = expanded(
        &mut b,
        &d,
        QualifiedName::new(
            Namespace::html(),
            None,
            DocumentString::from_utf16(vec![0x61, 0x72, 0xd800]),
        ),
    );
    let footer = expanded(
        &mut b,
        &d,
        QualifiedName::new(Namespace::html(), Some("f".into()), "footer"),
    );
    children(&mut b, &section, std::slice::from_ref(&uppercase));
    children(&mut b, &uppercase, std::slice::from_ref(&no_ns));
    children(&mut b, &no_ns, std::slice::from_ref(&malformed));
    children(&mut b, &malformed, std::slice::from_ref(&footer));
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&footer).unwrap())),
        &section_scope(&c, &section)
    );
}

#[test]
fn actual_document_body_is_selected_even_when_it_is_not_an_ancestor() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "html");
    let body = html(&mut b, &d, "body");
    let detached_body = html(&mut b, &d, "body");
    let footer = html(&mut b, &d, "footer");
    children(&mut b, &d, std::slice::from_ref(&root));
    children(&mut b, &root, std::slice::from_ref(&body));
    b.establish_root(&detached_body).unwrap();
    children(&mut b, &detached_body, std::slice::from_ref(&footer));
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&footer).unwrap())),
        &body_scope(&c, &d, Some(&body))
    );
}

#[test]
fn first_direct_body_or_frameset_wins_in_joint_source_order() {
    for names in [["body", "frameset"], ["frameset", "body"]] {
        let (mut b, d) = builder();
        let root = html(&mut b, &d, "html");
        let first = html(&mut b, &d, names[0]);
        let later = html(&mut b, &d, names[1]);
        let footer = html(&mut b, &d, "footer");
        b.establish_root(&footer).unwrap();
        children(&mut b, &d, std::slice::from_ref(&root));
        children(&mut b, &root, &[first.clone(), later]);
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&footer).unwrap())),
            &body_scope(&c, &d, Some(&first))
        );
    }
}

#[test]
fn direct_body_selection_ignores_wrapped_foreign_case_and_unavailable_descendants() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "html");
    let wrapper = html(&mut b, &d, "div");
    let nested = html(&mut b, &d, "body");
    children(&mut b, &wrapper, &[nested]);
    let foreign = expanded(
        &mut b,
        &d,
        QualifiedName::new(Namespace::svg(), None, "body"),
    );
    let upper = html(&mut b, &d, "BODY");
    let exact = expanded(
        &mut b,
        &d,
        QualifiedName::new(Namespace::html(), Some("p".into()), "frameset"),
    );
    let later = html(&mut b, &d, &"x".repeat(10000));
    let footer = html(&mut b, &d, "footer");
    b.establish_root(&footer).unwrap();
    children(&mut b, &d, std::slice::from_ref(&root));
    children(
        &mut b,
        &root,
        &[wrapper, foreign, upper, exact.clone(), later],
    );
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&footer).unwrap())),
        &body_scope(&c, &d, Some(&exact))
    );
}

#[test]
fn unique_non_html_document_element_is_null_but_competing_elements_are_unsupported() {
    for name in [
        QualifiedName::new(Namespace::html(), None, "div"),
        QualifiedName::new(Namespace::svg(), None, "html"),
        QualifiedName::new(Namespace::html(), None, "HTML"),
    ] {
        for with_later in [false, true] {
            let (mut b, d) = builder();
            let first = expanded(&mut b, &d, name.clone());
            let footer = html(&mut b, &d, "footer");
            b.establish_root(&footer).unwrap();
            if with_later {
                let later = html(&mut b, &d, "html");
                let body = html(&mut b, &d, "body");
                children(&mut b, &later, &[body]);
                children(&mut b, &d, &[first, later]);
            } else {
                children(&mut b, &d, std::slice::from_ref(&first));
            }
            let c = b.finish().unwrap();
            let expected = if with_later {
                HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::AmbiguousDocumentElement)
            } else {
                body_scope(&c, &d, None)
            };
            assert_eq!(
                value(&report(&c, &c.subject_id(&footer).unwrap())),
                &expected
            );
        }
    }
}

#[test]
fn document_element_skips_text_comments_pi_and_doctype_without_reading_payloads() {
    let (mut b, d) = builder();
    let text = b.add_text(&d, "unread".into()).unwrap();
    let comment = b.add_comment(&d, "unread".into()).unwrap();
    let pi = b
        .add_processing_instruction(&d, "unread".into(), "unread".into())
        .unwrap();
    let doctype = b
        .add_doctype(&d, HtmlDoctype::new("html", None, None))
        .unwrap();
    let root = html(&mut b, &d, "html");
    let body = html(&mut b, &d, "body");
    let footer = html(&mut b, &d, "footer");
    b.establish_root(&footer).unwrap();
    children(&mut b, &d, &[text, comment, pi, doctype, root.clone()]);
    children(&mut b, &root, std::slice::from_ref(&body));
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&footer).unwrap())),
        &body_scope(&c, &d, Some(&body))
    );
}

#[test]
fn complete_empty_document_or_body_candidate_list_establishes_null_not_missing() {
    for with_html in [false, true] {
        let (mut b, d) = builder();
        let footer = html(&mut b, &d, "footer");
        b.establish_root(&footer).unwrap();
        if with_html {
            let root = html(&mut b, &d, "html");
            children(&mut b, &d, std::slice::from_ref(&root));
            children(&mut b, &root, &[]);
        } else {
            children(&mut b, &d, &[]);
        }
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&footer).unwrap())),
            &body_scope(&c, &d, None)
        );
    }
}

#[test]
fn fallback_uses_the_subjects_associated_document_not_another_document_in_capture() {
    let (mut b, other) = builder();
    let d = b
        .add_document(
            HtmlDocumentMode::Quirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let wrong_root = html(&mut b, &other, "html");
    let wrong_body = html(&mut b, &other, "body");
    children(&mut b, &other, std::slice::from_ref(&wrong_root));
    children(&mut b, &wrong_root, &[wrong_body]);
    let root = html(&mut b, &d, "html");
    let actual = html(&mut b, &d, "frameset");
    children(&mut b, &d, std::slice::from_ref(&root));
    children(&mut b, &root, std::slice::from_ref(&actual));
    let footer = html(&mut b, &d, "footer");
    b.establish_root(&footer).unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&footer).unwrap())),
        &body_scope(&c, &d, Some(&actual))
    );
}

#[test]
fn obsolete_frameset_selection_is_exact_html_case_and_utf16() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "html");
    let upper = html(&mut b, &d, "FRAMESET");
    let foreign = expanded(
        &mut b,
        &d,
        QualifiedName::new(Namespace::svg(), None, "frameset"),
    );
    let malformed = expanded(
        &mut b,
        &d,
        QualifiedName::new(
            Namespace::html(),
            None,
            DocumentString::from_utf16(vec![102, 114, 97, 109, 101, 115, 101, 0xd800]),
        ),
    );
    let footer = html(&mut b, &d, "footer");
    b.establish_root(&footer).unwrap();
    children(&mut b, &d, std::slice::from_ref(&root));
    children(&mut b, &root, &[upper, foreign, malformed]);
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&footer).unwrap())),
        &body_scope(&c, &d, None)
    );
}

#[test]
fn first_body_witness_does_not_read_later_long_names_or_any_body_children() {
    let mut reports = Vec::new();
    for extra in [false, true] {
        let (mut b, d) = builder();
        let root = html(&mut b, &d, "html");
        let body = html(&mut b, &d, "body");
        let footer = html(&mut b, &d, "footer");
        b.establish_root(&footer).unwrap();
        children(&mut b, &d, std::slice::from_ref(&root));
        if extra {
            let later = html(&mut b, &d, &"x".repeat(10000));
            children(&mut b, &root, &[body.clone(), later]);
        } else {
            children(&mut b, &root, std::slice::from_ref(&body));
        }
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&footer).unwrap());
        assert_eq!(value(&r), &body_scope(&c, &d, Some(&body)));
        reports.push(r);
    }
    assert_eq!(reports[0].usage(), reports[1].usage());
}

#[test]
fn unavailable_parent_blocks_fallback_even_with_a_known_document_body() {
    for intermediate in [false, true] {
        let (mut b, d) = builder();
        let root = html(&mut b, &d, "html");
        let body = html(&mut b, &d, "body");
        children(&mut b, &d, std::slice::from_ref(&root));
        children(&mut b, &root, &[body]);
        let footer = html(&mut b, &d, "footer");
        let unknown = if intermediate {
            let wrapper = html(&mut b, &d, "div");
            children(&mut b, &wrapper, std::slice::from_ref(&footer));
            wrapper
        } else {
            footer.clone()
        };
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&footer).unwrap())),
            &HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(
                c.subject_id(&unknown).unwrap()
            ))
        );
    }
}

#[test]
fn unavailable_document_or_html_child_order_is_influencing_missing() {
    for with_html in [false, true] {
        let (mut b, d) = builder();
        let footer = html(&mut b, &d, "footer");
        b.establish_root(&footer).unwrap();
        let missing = if with_html {
            let root = html(&mut b, &d, "html");
            children(&mut b, &d, std::slice::from_ref(&root));
            root
        } else {
            d.clone()
        };
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&footer).unwrap())),
            &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(
                c.subject_id(&missing).unwrap()
            ))
        );
    }
}

#[test]
fn template_and_shadow_hosts_do_not_supply_sectioning_ancestry() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let root = html(&mut b, &d, "html");
        let body = html(&mut b, &d, "body");
        children(&mut b, &d, std::slice::from_ref(&root));
        children(&mut b, &root, std::slice::from_ref(&body));
        let section = html(&mut b, &d, "article");
        let host = html(&mut b, &d, if shadow { "div" } else { "template" });
        children(&mut b, &section, std::slice::from_ref(&host));
        let f = if shadow {
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
        } else {
            let f = b.add_fragment(&d).unwrap();
            b.establish_template_contents(&host, &f).unwrap();
            f
        };
        let footer = html(&mut b, &d, "footer");
        children(&mut b, &f, std::slice::from_ref(&footer));
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&footer).unwrap())),
            &body_scope(&c, &d, Some(&body))
        );
    }
}

#[test]
fn wrong_roles_non_elements_and_foreign_footer_preserve_applicability() {
    let (mut b, d) = builder();
    let header = html(&mut b, &d, "header");
    let upper = html(&mut b, &d, "FOOTER");
    let text = b.add_text(&d, "footer".into()).unwrap();
    let foreign = expanded(
        &mut b,
        &d,
        QualifiedName::new(Namespace::svg(), None, "footer"),
    );
    let c = b.finish().unwrap();
    for subject in [&header, &upper, &text, &d] {
        assert_eq!(
            value(&report(&c, &c.subject_id(subject).unwrap())),
            &HtmlSemanticValue::Inapplicable
        );
    }
    assert_eq!(
        value(&report(&c, &c.subject_id(&foreign).unwrap())),
        &HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
    );
}

#[test]
fn huge_unconsulted_attributes_and_text_succeed_at_zero_raw_scratch_and_nested_caps() {
    let (mut b, d) = builder();
    let section = html(&mut b, &d, "article");
    let footer = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), None, "footer"),
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), None, "title"),
                "x".repeat(10000),
            )])
            .unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let text = b.add_text(&d, "x".repeat(10000).into()).unwrap();
    children(&mut b, &section, std::slice::from_ref(&footer));
    children(&mut b, &footer, &[text]);
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&footer).unwrap());
    assert_eq!(value(&r), &section_scope(&c, &section));
    assert_eq!(r.usage().scratch_records(), 0);
    assert_eq!(r.usage().peak_scratch_records(), 0);
    assert_eq!(r.usage().nested_output_records(), 0);
}

#[test]
fn exact_and_one_short_work_output_and_observed_bytes_preserve_qualified_first_stop() {
    let (c, id, owner) = simple();
    let cx = context(&c);
    let u = report(&c, &id).usage();
    assert_eq!(u.output_records(), 1);
    assert_eq!(
        u.allocated_bytes(),
        std::mem::size_of::<HtmlSemanticResult>() * u.output_records()
    );
    for (resource, exact) in [
        (HtmlSemanticResource::Work, u.work()),
        (HtmlSemanticResource::OutputRecords, u.output_records()),
        (HtmlSemanticResource::AllocatedBytes, u.allocated_bytes()),
    ] {
        let req = request(&id);
        let r = session(
            &c,
            &cx,
            resource_limits(resource, exact),
            0,
            HtmlSemanticCancellation::new(),
        )
        .query(std::slice::from_ref(&req))
        .unwrap();
        assert_eq!(
            value(&r),
            &HtmlSemanticValue::Known(HtmlFooterScope::Section(owner.clone()))
        );
        let mut s = session(
            &c,
            &cx,
            resource_limits(resource, exact - 1),
            0,
            HtmlSemanticCancellation::new(),
        );
        let f = s.query(std::slice::from_ref(&req)).unwrap_err();
        assert!(
            matches!(f.kind(), HtmlSemanticFailureKind::LimitExceeded {resource: actual, ..} if *actual == resource)
        );
        assert_eq!(s.query(&[]).unwrap_err(), f);
        assert_eq!(s.usage().scratch_records(), 0);
        assert_eq!(f.qualification().unwrap().capture(), c.identity());
    }
}

#[test]
fn fallback_work_admission_is_exact_and_terminal_failure_is_not_null_or_missing() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "html");
    let body = html(&mut b, &d, "body");
    let footer = html(&mut b, &d, "footer");
    b.establish_root(&footer).unwrap();
    children(&mut b, &d, std::slice::from_ref(&root));
    children(&mut b, &root, std::slice::from_ref(&body));
    let c = b.finish().unwrap();
    let id = c.subject_id(&footer).unwrap();
    let exact = report(&c, &id).usage().work();
    let cx = context(&c);
    let r = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::Work, exact),
        0,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(&id)])
    .unwrap();
    assert_eq!(value(&r), &body_scope(&c, &d, Some(&body)));
    let mut s = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::Work, exact - 1),
        0,
        HtmlSemanticCancellation::new(),
    );
    let f = s.query(&[request(&id)]).unwrap_err();
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            ..
        }
    ));
    assert_eq!(s.query(&[request(&id)]).unwrap_err(), f);
}

#[test]
fn a_section_witness_does_not_spend_fallback_or_later_ancestor_work() {
    let (baseline, bid, _) = simple();
    let exact = report(&baseline, &bid).usage().work();
    let (mut b, d) = builder();
    let huge = html(&mut b, &d, &"x".repeat(10000));
    let section = html(&mut b, &d, "section");
    let footer = html(&mut b, &d, "footer");
    children(&mut b, &huge, std::slice::from_ref(&section));
    children(&mut b, &section, std::slice::from_ref(&footer));
    let c = b.finish().unwrap();
    let r = session(
        &c,
        &context(&c),
        resource_limits(HtmlSemanticResource::Work, exact),
        0,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(&c.subject_id(&footer).unwrap())])
    .unwrap();
    assert_eq!(value(&r), &section_scope(&c, &section));
}

#[test]
fn repeated_queries_charge_again_and_late_batch_failure_discards_the_prefix() {
    let (c, id, owner) = simple();
    let cx = context(&c);
    let req = request(&id);
    let mut s = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let old = s.query(std::slice::from_ref(&req)).unwrap();
    let next = s.query(std::slice::from_ref(&req)).unwrap();
    assert_eq!(next.usage().work(), 2 * old.usage().work());
    assert_eq!(
        next.usage().allocated_bytes(),
        2 * old.usage().allocated_bytes()
    );
    let mut bounded = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::Work, old.usage().work()),
        0,
        HtmlSemanticCancellation::new(),
    );
    let f = bounded.query(&[req.clone(), req]).unwrap_err();
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            ..
        }
    ));
    assert_eq!(bounded.query(&[]).unwrap_err(), f);
    assert_eq!(bounded.usage().scratch_records(), 0);
    assert_eq!(
        value(&old),
        &HtmlSemanticValue::Known(HtmlFooterScope::Section(owner))
    );
    assert_eq!(value(&old), value(&next));
}

#[test]
fn cancellation_is_sticky_and_completed_scope_remains_available() {
    let (c, id, owner) = simple();
    let cx = context(&c);
    let cancel = HtmlSemanticCancellation::new();
    let mut s = session(&c, &cx, limits(), 0, cancel.clone());
    let old = s.query(&[request(&id)]).unwrap();
    cancel.cancel();
    let f = s.query(&[request(&id)]).unwrap_err();
    assert_eq!(f.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(f.qualification().unwrap(), old.qualification());
    assert_eq!(s.query(&[]).unwrap_err(), f);
    assert_eq!(
        value(&old),
        &HtmlSemanticValue::Known(HtmlFooterScope::Section(owner))
    );
    assert_eq!(s.usage().scratch_records(), 0);
}

#[test]
fn changed_ancestry_host_and_context_qualify_conservative_recomputation() {
    let mut returned = Vec::new();
    for section in [true, false] {
        let (mut b, d) = builder();
        let root = html(&mut b, &d, "html");
        let body = html(&mut b, &d, "body");
        let ancestor = html(&mut b, &d, if section { "section" } else { "div" });
        let footer = html(&mut b, &d, "footer");
        children(&mut b, &d, std::slice::from_ref(&root));
        children(&mut b, &root, std::slice::from_ref(&body));
        children(&mut b, &body, std::slice::from_ref(&ancestor));
        children(&mut b, &ancestor, std::slice::from_ref(&footer));
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&footer).unwrap());
        let expected = if section {
            section_scope(&c, &ancestor)
        } else {
            body_scope(&c, &d, Some(&body))
        };
        assert_eq!(value(&r), &expected);
        assert_eq!(
            r.dependencies(),
            HtmlSemanticDependencies::ConservativeRecompute
        );
        returned.push((r, expected));
    }
    assert_ne!(returned[0].0.capture(), returned[1].0.capture());
    assert_ne!(returned[0].0.host_revision(), returned[1].0.host_revision());
    assert_eq!(value(&returned[0].0), &returned[0].1);
    let (c, id, _) = simple();
    let cx = context(&c);
    let a = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
        .query(&[request(&id)])
        .unwrap();
    let other_cx = context(&c);
    let b = session(
        &c,
        &other_cx,
        resource_limits(HtmlSemanticResource::Work, 20000001),
        1,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(&id)])
    .unwrap();
    assert_ne!(a.context(), b.context());
    assert_ne!(a.limits(), b.limits());
    assert_eq!(b.attribute_input_limit(), 1);
    assert_eq!(value(&a), value(&b));
}

#[test]
fn changed_body_order_has_distinct_qualification_and_preserves_old_scope_ids() {
    let mut reports = Vec::new();
    for reversed in [false, true] {
        let (mut b, d) = builder();
        let root = html(&mut b, &d, "html");
        let body = html(&mut b, &d, "body");
        let frameset = html(&mut b, &d, "frameset");
        let footer = html(&mut b, &d, "footer");
        b.establish_root(&footer).unwrap();
        children(&mut b, &d, std::slice::from_ref(&root));
        let order = if reversed {
            [frameset.clone(), body.clone()]
        } else {
            [body.clone(), frameset.clone()]
        };
        children(&mut b, &root, &order);
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&footer).unwrap());
        let expected = body_scope(&c, &d, Some(if reversed { &frameset } else { &body }));
        assert_eq!(value(&r), &expected);
        reports.push((r, expected));
    }
    assert_ne!(reports[0].0.qualification(), reports[1].0.qualification());
    assert_eq!(value(&reports[0].0), &reports[0].1);
    assert_eq!(value(&reports[1].0), &reports[1].1);
}

#[test]
fn nonterminal_missing_is_a_complete_fact_and_does_not_stop_a_later_scope() {
    let (mut b, d) = builder();
    let missing = html(&mut b, &d, "footer");
    let section = html(&mut b, &d, "article");
    let footer = html(&mut b, &d, "footer");
    children(&mut b, &section, std::slice::from_ref(&footer));
    let c = b.finish().unwrap();
    let missing_id = c.subject_id(&missing).unwrap();
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let r = s
        .query(&[
            request(&missing_id),
            request(&c.subject_id(&footer).unwrap()),
        ])
        .unwrap();
    assert_eq!(r.results().len(), 2);
    assert_eq!(
        value(&r),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(missing_id))
    );
    assert_eq!(
        r.results()[1].fact(),
        &HtmlSemanticFact::FooterScope(section_scope(&c, &section))
    );
    assert!(s.failure().is_none());
    assert_eq!(s.usage().scratch_records(), 0);
}

#[test]
fn foreign_capture_subjects_and_contexts_are_rejected_without_a_batch_prefix() {
    let (c, id, _) = simple();
    let (other, oid, _) = simple();
    let cx = context(&c);
    assert!(
        HtmlSemanticSession::new(
            &c,
            &context(&other),
            HtmlValueInterpreter::new(0),
            limits(),
            HtmlSemanticCancellation::new()
        )
        .is_err()
    );
    let mut s = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let f = s.query(&[request(&id), request(&oid)]).unwrap_err();
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::Capture(HtmlCaptureError::ForeignCapture)
    ));
    assert_eq!(s.query(&[]).unwrap_err(), f);
    assert_eq!(f.qualification().unwrap().capture(), c.identity());
}

#[test]
fn f1_multiple_document_elements_do_not_supply_a_body_from_the_first_html_root() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "html");
    let body = html(&mut b, &d, "body");
    let second = html(&mut b, &d, "div");
    let footer = html(&mut b, &d, "footer");
    b.establish_root(&footer).unwrap();
    children(&mut b, &d, &[root.clone(), second]);
    children(&mut b, &root, std::slice::from_ref(&body));
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&footer).unwrap())),
        &HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::AmbiguousDocumentElement)
    );
}

#[test]
fn f1_wrong_first_element_does_not_hide_document_element_ambiguity() {
    for name in [
        QualifiedName::new(Namespace::html(), None, "div"),
        QualifiedName::new(Namespace::svg(), None, "html"),
    ] {
        let (mut b, d) = builder();
        let first = expanded(&mut b, &d, name);
        let later = html(&mut b, &d, "html");
        let body = html(&mut b, &d, "body");
        let footer = html(&mut b, &d, "footer");
        b.establish_root(&footer).unwrap();
        children(&mut b, &d, &[first, later.clone()]);
        children(&mut b, &later, std::slice::from_ref(&body));
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&footer).unwrap())),
            &HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::AmbiguousDocumentElement)
        );
    }
}

#[test]
fn f1_nearest_section_witness_bypasses_irrelevant_document_ambiguity() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "html");
    let second = html(&mut b, &d, "div");
    let section = html(&mut b, &d, "article");
    let footer = html(&mut b, &d, "footer");
    children(&mut b, &d, &[root.clone(), second]);
    children(&mut b, &root, std::slice::from_ref(&section));
    children(&mut b, &section, std::slice::from_ref(&footer));
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&footer).unwrap())),
        &section_scope(&c, &section)
    );
}
