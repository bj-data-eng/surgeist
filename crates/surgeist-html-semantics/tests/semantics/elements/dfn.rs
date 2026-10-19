use crate::semantics::support::builder;
use crate::semantics::support::context;
use crate::semantics::support::session;
use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedChildIds, HtmlCapturedIsValue, HtmlCapturedShadowProperties,
    HtmlDefiningTerm, HtmlDefiningTermOrigin, HtmlSemanticCancellation, HtmlSemanticDependencies,
    HtmlSemanticFact, HtmlSemanticFailureKind, HtmlSemanticLimits, HtmlSemanticMissing,
    HtmlSemanticQuery, HtmlSemanticReport, HtmlSemanticRequest, HtmlSemanticResource,
    HtmlSemanticResult, HtmlSemanticSession, HtmlSemanticUnsupported, HtmlSemanticValue,
};

fn attr(local: &str, value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(QualifiedName::new(Namespace::none(), None, local), value)
}
fn expanded(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    name: QualifiedName,
    attrs: Vec<Attribute>,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        name,
        OrderedAttributes::new(attrs).unwrap(),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    local: &str,
    attrs: Vec<Attribute>,
) -> HtmlCaptureBuilderSubject {
    expanded(
        b,
        d,
        QualifiedName::new(Namespace::html(), None, local),
        attrs,
    )
}
fn text(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    value: impl Into<DocumentString>,
) -> HtmlCaptureBuilderSubject {
    b.add_text(d, value.into()).unwrap()
}
fn children(
    b: &mut HtmlCaptureBuilder,
    p: &HtmlCaptureBuilderSubject,
    list: &[HtmlCaptureBuilderSubject],
) {
    b.establish_children(p, list).unwrap();
}

fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(1000000, 100, 1000, 0, 1000000, 10000)
}

fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::DfnDefiningTerm)
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
fn value(r: &HtmlSemanticReport) -> &HtmlSemanticValue<HtmlDefiningTerm> {
    match r.results()[0].fact() {
        HtmlSemanticFact::DfnDefiningTerm(v) => v,
        other => panic!("unexpected fact: {other:?}"),
    }
}
fn known(r: &HtmlSemanticReport) -> &HtmlDefiningTerm {
    match value(r) {
        HtmlSemanticValue::Known(v) => v,
        other => panic!("unexpected term: {other:?}"),
    }
}
fn own(raw: impl Into<DocumentString>) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "dfn", vec![attr("title", raw)]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    (c, id)
}
fn fallback(raw: impl Into<DocumentString>) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "dfn", vec![]);
    let t = text(&mut b, &d, raw);
    children(&mut b, &e, &[t]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    (c, id)
}

#[test]
fn own_title_is_exact_including_empty_utf16_and_unknown_topology() {
    for raw in [vec![], vec![32, 65, 13, 10, 0, 0xd800, 0xdc00, 32]] {
        let (c, id) = own(DocumentString::from_utf16(raw.clone()));
        let r = report(&c, &id, raw.len());
        assert_eq!(known(&r).value().code_units(), raw);
        assert_eq!(known(&r).origin(), &HtmlDefiningTermOrigin::OwnTitle);
        assert_eq!(r.usage().nested_output_records(), 0);
        assert_eq!(r.usage().peak_scratch_records(), 0);
        assert_eq!(
            r.usage().allocated_bytes(),
            r.usage().output_records() * std::mem::size_of::<HtmlSemanticResult>()
        );
    }
}

#[test]
fn own_title_wins_over_abbreviation_title_and_descendant_text() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "dfn", vec![attr("title", "own")]);
    let a = element(
        &mut b,
        &d,
        "abbr",
        vec![attr("title", "oversized unselected")],
    );
    let t = text(&mut b, &d, "not the term");
    children(&mut b, &a, &[t]);
    children(&mut b, &e, &[a]);
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&e).unwrap(), 3);
    assert_eq!(known(&r).value().code_units(), [111, 119, 110]);
    assert_eq!(known(&r).origin(), &HtmlDefiningTermOrigin::OwnTitle);
}

#[test]
fn one_abbreviation_with_comments_and_pi_selects_exact_title_without_descending() {
    for raw in [vec![], vec![32, 13, 10, 0xd800, 65, 0]] {
        let (mut b, d) = builder();
        let e = element(&mut b, &d, "dfn", vec![]);
        let a = element(
            &mut b,
            &d,
            "abbr",
            vec![attr("title", DocumentString::from_utf16(raw.clone()))],
        );
        let comment = b.add_comment(&d, "not Text".into()).unwrap();
        let pi = b
            .add_processing_instruction(&d, "x".into(), "not Text".into())
            .unwrap();
        children(&mut b, &e, &[comment, a.clone(), pi]);
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&e).unwrap(), raw.len());
        assert_eq!(known(&r).value().code_units(), raw);
        assert_eq!(
            known(&r).origin(),
            &HtmlDefiningTermOrigin::AbbreviationTitle(c.subject_id(&a).unwrap())
        );
        assert_eq!(r.usage().nested_output_records(), 0);
        assert_eq!(r.usage().peak_scratch_records(), 0);
    }
}

#[test]
fn even_empty_or_whitespace_direct_text_defeats_abbreviation_title() {
    for raw in ["", " ", "\n"] {
        let (mut b, d) = builder();
        let e = element(&mut b, &d, "dfn", vec![]);
        let a = element(&mut b, &d, "abbr", vec![attr("title", "oversized unused")]);
        let inside = text(&mut b, &d, "A");
        children(&mut b, &a, &[inside]);
        let direct = text(&mut b, &d, raw);
        children(&mut b, &e, &[a, direct]);
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&e).unwrap(), 0);
        assert_eq!(known(&r).value().to_utf8().unwrap(), format!("A{raw}"));
        assert_eq!(known(&r).origin(), &HtmlDefiningTermOrigin::DescendantText);
    }
}

#[test]
fn second_element_defeats_abbreviation_title_without_admitting_it() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "dfn", vec![]);
    let a = element(&mut b, &d, "abbr", vec![attr("title", "oversized unused")]);
    let t = text(&mut b, &d, "A");
    children(&mut b, &a, &[t]);
    let s = element(&mut b, &d, "span", vec![]);
    let t = text(&mut b, &d, "B");
    children(&mut b, &s, &[t]);
    children(&mut b, &e, &[a, s]);
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&e).unwrap(), 0);
    assert_eq!(known(&r).value().code_units(), [65, 66]);
    assert_eq!(known(&r).origin(), &HtmlDefiningTermOrigin::DescendantText);
}

#[test]
fn absent_abbreviation_title_and_other_element_roles_use_descendant_text() {
    for (ns, local) in [
        (Namespace::html(), "abbr"),
        (Namespace::html(), "span"),
        (Namespace::html(), "ABBR"),
        (Namespace::from_uri("urn:x"), "abbr"),
    ] {
        let (mut b, d) = builder();
        let e = element(&mut b, &d, "dfn", vec![]);
        let attrs = if local == "abbr" && ns == Namespace::html() {
            vec![]
        } else {
            vec![attr("title", "oversized unused")]
        };
        let child = expanded(&mut b, &d, QualifiedName::new(ns, None, local), attrs);
        let t = text(&mut b, &d, "fallback");
        children(&mut b, &child, &[t]);
        children(&mut b, &e, &[child]);
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&e).unwrap(), 0);
        assert_eq!(known(&r).value().to_utf8().unwrap(), "fallback");
        assert_eq!(known(&r).origin(), &HtmlDefiningTermOrigin::DescendantText);
    }
}

#[test]
fn exact_title_namespace_and_local_name_apply_but_prefix_is_not_part_of_identity() {
    let (mut b, d) = builder();
    let e = element(
        &mut b,
        &d,
        "dfn",
        vec![
            Attribute::new(
                QualifiedName::new(Namespace::from_uri("urn:x"), None, "title"),
                "unused",
            ),
            attr("TITLE", "unused"),
            Attribute::new(
                QualifiedName::new(Namespace::none(), Some("p".into()), "title"),
                "exact",
            ),
        ],
    );
    let c = b.finish().unwrap();
    assert_eq!(
        known(&report(&c, &c.subject_id(&e).unwrap(), 5))
            .value()
            .to_utf8()
            .unwrap(),
        "exact"
    );
    let (mut b, d) = builder();
    let e = expanded(
        &mut b,
        &d,
        QualifiedName::new(Namespace::html(), Some("p".into()), "dfn"),
        vec![],
    );
    let a = expanded(
        &mut b,
        &d,
        QualifiedName::new(Namespace::html(), Some("q".into()), "abbr"),
        vec![
            Attribute::new(
                QualifiedName::new(Namespace::from_uri("urn:x"), None, "title"),
                "unused",
            ),
            Attribute::new(
                QualifiedName::new(Namespace::none(), Some("r".into()), "title"),
                "child",
            ),
        ],
    );
    children(&mut b, &e, std::slice::from_ref(&a));
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&e).unwrap(), 5);
    assert_eq!(known(&r).value().to_utf8().unwrap(), "child");
    assert_eq!(
        known(&r).origin(),
        &HtmlDefiningTermOrigin::AbbreviationTitle(c.subject_id(&a).unwrap())
    );
}

#[test]
fn descendant_fallback_keeps_script_img_foreign_text_and_exact_utf16_without_separators() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "dfn", vec![]);
    let first = text(&mut b, &d, " A\r\n");
    let mut list = vec![first];
    for (ns, local, raw) in [
        (Namespace::html(), "script", "S"),
        (Namespace::html(), "img", "I"),
        (Namespace::from_uri("urn:x"), "foreign", "F"),
    ] {
        let p = expanded(
            &mut b,
            &d,
            QualifiedName::new(ns, None, local),
            vec![attr("alt", "unused")],
        );
        let t = text(&mut b, &d, raw);
        children(&mut b, &p, &[t]);
        list.push(p);
    }
    let br = element(&mut b, &d, "br", vec![]);
    children(&mut b, &br, &[]);
    list.push(br);
    list.push(text(
        &mut b,
        &d,
        DocumentString::from_utf16(vec![0xa0, 0, 0xd800, 0xdc00]),
    ));
    list.push(b.add_comment(&d, "unused".into()).unwrap());
    list.push(
        b.add_processing_instruction(&d, "p".into(), "unused".into())
            .unwrap(),
    );
    children(&mut b, &e, &list);
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&e).unwrap(), 0);
    assert_eq!(
        known(&r).value().code_units(),
        [32, 65, 13, 10, 83, 73, 70, 0xa0, 0, 0xd800, 0xdc00]
    );
    assert_eq!(known(&r).origin(), &HtmlDefiningTermOrigin::DescendantText);
    assert_eq!(r.usage().scratch_records(), 0);
}

#[test]
fn ancestor_title_never_substitutes_for_an_empty_detached_term() {
    let (mut b, d) = builder();
    let p = element(
        &mut b,
        &d,
        "div",
        vec![attr("title", "oversized unused ancestor")],
    );
    let e = element(&mut b, &d, "dfn", vec![]);
    children(&mut b, &e, &[]);
    children(&mut b, &p, std::slice::from_ref(&e));
    b.establish_root(&p).unwrap();
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&e).unwrap(), 0);
    assert!(known(&r).value().is_empty());
    assert_eq!(known(&r).origin(), &HtmlDefiningTermOrigin::DescendantText);
    assert_eq!(r.usage().peak_scratch_records(), 0);
    assert_eq!(r.usage().nested_output_records(), 0);
}

#[test]
fn missing_direct_or_influencing_descendant_children_is_not_known_empty() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "dfn", vec![]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    assert_eq!(
        value(&report(&c, &id, 0)),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(id))
    );
    for local in ["span", "abbr"] {
        let (mut b, d) = builder();
        let e = element(&mut b, &d, "dfn", vec![]);
        let p = element(&mut b, &d, local, vec![]);
        children(&mut b, &e, std::slice::from_ref(&p));
        let c = b.finish().unwrap();
        let id = c.subject_id(&p).unwrap();
        let r = report(&c, &c.subject_id(&e).unwrap(), 0);
        assert_eq!(
            value(&r),
            &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(id))
        );
        assert_eq!(r.usage().scratch_records(), 0);
    }
}

#[test]
fn non_elements_wrong_html_roles_and_foreign_subjects_remain_distinct() {
    let (mut b, d) = builder();
    let t = text(&mut b, &d, "x");
    let f = b.add_fragment(&d).unwrap();
    let p = element(&mut b, &d, "p", vec![attr("title", "oversized unused")]);
    let upper = element(&mut b, &d, "DFN", vec![attr("title", "oversized unused")]);
    let foreign = expanded(
        &mut b,
        &d,
        QualifiedName::new(Namespace::from_uri("urn:x"), None, "dfn"),
        vec![attr("title", "unused")],
    );
    let c = b.finish().unwrap();
    for x in [d, t, f, p, upper] {
        assert_eq!(
            value(&report(&c, &c.subject_id(&x).unwrap(), 0)),
            &HtmlSemanticValue::Inapplicable
        );
    }
    assert_eq!(
        value(&report(&c, &c.subject_id(&foreign).unwrap(), 0)),
        &HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
    );
    assert_eq!(
        HtmlSemanticQuery::DfnDefiningTerm.source_clause(),
        "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#defining-term"
    );
}

#[test]
fn selected_own_and_abbreviation_titles_require_the_whole_raw_cap() {
    for abbreviation in [false, true] {
        let (mut b, d) = builder();
        let e = element(
            &mut b,
            &d,
            "dfn",
            if abbreviation {
                vec![]
            } else {
                vec![attr("title", " AB ")]
            },
        );
        if abbreviation {
            let a = element(&mut b, &d, "abbr", vec![attr("title", " AB ")]);
            children(&mut b, &e, &[a]);
        }
        let c = b.finish().unwrap();
        let id = c.subject_id(&e).unwrap();
        let cx = context(&c);
        assert_eq!(
            known(&report(&c, &id, 4)).value().code_units(),
            [32, 65, 66, 32]
        );
        let mut s = session(&c, &cx, limits(), 3, HtmlSemanticCancellation::new());
        let failure = s.query(&[request(&id)]).unwrap_err();
        assert!(matches!(
            failure.kind(),
            HtmlSemanticFailureKind::Attribute(_)
        ));
        assert_eq!(failure.qualification().unwrap().attribute_input_limit(), 3);
        assert_eq!(s.query(&[]).unwrap_err(), failure);
        assert_eq!(s.usage().scratch_records(), 0);
        assert_eq!(s.usage().nested_output_records(), 0);
    }
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
            limits().output_records()
        },
        if resource == HtmlSemanticResource::ScratchRecords {
            cap
        } else {
            limits().scratch_records()
        },
        0,
        if resource == HtmlSemanticResource::AllocatedBytes {
            cap
        } else {
            limits().allocated_bytes()
        },
        if resource == HtmlSemanticResource::NestedOutputRecords {
            cap
        } else {
            limits().nested_output_records()
        },
    )
}

#[test]
fn exact_and_one_short_actual_text_storage_work_and_batch_caps_are_terminal() {
    let (c, id) = fallback("AB");
    let cx = context(&c);
    let r = report(&c, &id, 0);
    let u = r.usage();
    assert_eq!(known(&r).value().code_units(), [65, 66]);
    assert_eq!(u.nested_output_records(), 2);
    assert_eq!(u.peak_scratch_records(), 1);
    assert_eq!(
        u.allocated_bytes(),
        std::mem::size_of::<HtmlSemanticResult>() * u.output_records()
            + std::mem::size_of::<HtmlCapturedChildIds<'_>>()
            + 2 * std::mem::size_of::<u16>()
    );
    for (resource, exact) in [
        (HtmlSemanticResource::Work, u.work()),
        (HtmlSemanticResource::OutputRecords, u.output_records()),
        (
            HtmlSemanticResource::ScratchRecords,
            u.peak_scratch_records(),
        ),
        (
            HtmlSemanticResource::NestedOutputRecords,
            u.nested_output_records(),
        ),
        (HtmlSemanticResource::AllocatedBytes, u.allocated_bytes()),
    ] {
        let mut s = session(
            &c,
            &cx,
            resource_limits(resource, exact),
            0,
            HtmlSemanticCancellation::new(),
        );
        assert_eq!(
            known(&s.query(&[request(&id)]).unwrap())
                .value()
                .code_units(),
            [65, 66]
        );
        let mut s = session(
            &c,
            &cx,
            resource_limits(resource, exact - 1),
            0,
            HtmlSemanticCancellation::new(),
        );
        let f = s.query(&[request(&id)]).unwrap_err();
        assert!(
            matches!(f.kind(), HtmlSemanticFailureKind::LimitExceeded {resource:r,..} if *r == resource)
        );
        assert_eq!(s.usage().scratch_records(), 0);
        assert_eq!(
            f.qualification().unwrap().limits(),
            resource_limits(resource, exact - 1)
        );
        assert_eq!(s.query(&[]).unwrap_err(), f);
    }
}

#[test]
fn repeated_queries_and_duplicate_requests_charge_actual_materialization_cumulatively() {
    let (c, id) = fallback("ABC");
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let req = request(&id);
    let first = s.query(std::slice::from_ref(&req)).unwrap();
    let u = first.usage();
    let second = s.query(std::slice::from_ref(&req)).unwrap();
    assert_eq!(second.usage().allocated_bytes(), 2 * u.allocated_bytes());
    assert_eq!(
        second.usage().nested_output_records(),
        2 * u.nested_output_records()
    );
    assert_eq!(second.usage().output_records(), 2 * u.output_records());
    assert_eq!(second.usage().work(), 2 * u.work());
    assert_eq!(second.usage().scratch_records(), 0);
    assert_eq!(
        second.usage().peak_scratch_records(),
        u.peak_scratch_records()
    );
    let mut s = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let batch = s.query(&[request(&id), request(&id)]).unwrap();
    assert_eq!(batch.results().len(), 2);
    assert_eq!(
        batch.usage().nested_output_records(),
        2 * u.nested_output_records()
    );
    assert_eq!(known(&first).value().code_units(), [65, 66, 67]);
}

#[test]
fn late_raw_failure_discards_the_whole_batch_and_preserves_prior_reports() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "dfn", vec![]);
    let t = text(&mut b, &d, "AB");
    children(&mut b, &e, &[t]);
    let late = element(&mut b, &d, "dfn", vec![attr("title", "long")]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let id = c.subject_id(&e).unwrap();
    let mut s = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let earlier = s.query(&[request(&id)]).unwrap();
    let f = s
        .query(&[request(&id), request(&c.subject_id(&late).unwrap())])
        .unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(s.usage().nested_output_records(), 4);
    assert_eq!(s.usage().scratch_records(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), f);
    assert_eq!(known(&earlier).value().code_units(), [65, 66]);
}

#[test]
fn cancellation_before_execution_and_after_prior_success_is_sticky_without_new_work() {
    let (c, id) = fallback("ABC");
    let cx = context(&c);
    for prior in [false, true] {
        let cancel = HtmlSemanticCancellation::new();
        let mut s = session(&c, &cx, limits(), 0, cancel.clone());
        let earlier = prior.then(|| s.query(&[request(&id)]).unwrap());
        let usage = s.usage();
        cancel.cancel();
        let f = s.query(&[request(&id)]).unwrap_err();
        assert_eq!(f.kind(), &HtmlSemanticFailureKind::Cancelled);
        assert_eq!(f.qualification().unwrap().capture(), c.identity());
        assert_eq!(s.usage(), usage);
        assert_eq!(s.usage().scratch_records(), 0);
        assert_eq!(s.query(&[]).unwrap_err(), f);
        if let Some(earlier) = earlier {
            assert_eq!(known(&earlier).value().code_units(), [65, 66, 67]);
        }
    }
}

#[test]
fn capture_context_host_limits_and_raw_configuration_qualify_reports_and_stops() {
    let (c, id) = own("x");
    let cx = context(&c);
    let other_context = context(&c);
    let mut s = session(&c, &cx, limits(), 1, HtmlSemanticCancellation::new());
    let r = s.query(&[request(&id)]).unwrap();
    assert_eq!(r.qualification().capture(), c.identity());
    assert_eq!(r.qualification().context(), cx.identity());
    assert_ne!(r.qualification().context(), other_context.identity());
    assert_eq!(r.qualification().host_revision(), c.host_facts_revision());
    assert_eq!(r.qualification().limits(), limits());
    assert_eq!(r.qualification().attribute_input_limit(), 1);
    let (other, foreign) = own("y");
    let f = s.query(&[request(&foreign)]).unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Capture(_)));
    assert_eq!(f.qualification().unwrap(), r.qualification());
    assert_eq!(s.query(&[]).unwrap_err(), f);
    assert!(
        matches!(HtmlSemanticSession::new(&other, &cx, HtmlValueInterpreter::new(1), limits(),
        HtmlSemanticCancellation::new()), Err(f) if *f.kind() == HtmlSemanticFailureKind::ContextCaptureMismatch)
    );
}

#[test]
fn descendant_text_does_not_cross_template_contents_or_shadow_roots() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let e = element(&mut b, &d, "dfn", vec![]);
        let host = element(&mut b, &d, if shadow { "div" } else { "template" }, vec![]);
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
        let excluded = text(&mut b, &d, "excluded");
        children(&mut b, &f, &[excluded]);
        let ordinary = text(&mut b, &d, "ordinary");
        children(&mut b, &host, &[ordinary]);
        children(&mut b, &e, &[host]);
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&e).unwrap(), 0);
        assert_eq!(known(&r).value().to_utf8().unwrap(), "ordinary");
        assert_eq!(known(&r).origin(), &HtmlDefiningTermOrigin::DescendantText);
    }
}

#[test]
fn changed_title_child_predicate_and_text_recompute_under_distinct_capture_keys() {
    let mut reports = Vec::new();
    for (title, direct_text, expected, origin) in [
        (Some("own"), false, "own", 0),
        (None, false, "abbr", 1),
        (None, true, "literal", 2),
    ] {
        let (mut b, d) = builder();
        let e = element(
            &mut b,
            &d,
            "dfn",
            title.map(|v| attr("title", v)).into_iter().collect(),
        );
        let a = element(&mut b, &d, "abbr", vec![attr("title", "abbr")]);
        let inner = text(&mut b, &d, "literal");
        children(&mut b, &a, &[inner]);
        let mut list = vec![a.clone()];
        if direct_text {
            list.push(text(&mut b, &d, ""));
        }
        children(&mut b, &e, &list);
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&e).unwrap(), 4);
        assert_eq!(known(&r).value().to_utf8().unwrap(), expected);
        assert_eq!(
            known(&r).origin(),
            &match origin {
                0 => HtmlDefiningTermOrigin::OwnTitle,
                1 => HtmlDefiningTermOrigin::AbbreviationTitle(c.subject_id(&a).unwrap()),
                _ => HtmlDefiningTermOrigin::DescendantText,
            }
        );
        assert_eq!(
            r.dependencies(),
            HtmlSemanticDependencies::ConservativeRecompute
        );
        reports.push(r);
    }
    for pair in reports.windows(2) {
        assert_ne!(pair[0].capture(), pair[1].capture());
        assert_ne!(pair[0].host_revision(), pair[1].host_revision());
        assert_ne!(
            pair[0].qualification().context(),
            pair[1].qualification().context()
        );
    }
}

#[test]
fn missing_after_partial_text_retains_cumulative_capacity_but_releases_live_scratch() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "dfn", vec![]);
    let t = text(&mut b, &d, "AB");
    let gap = element(&mut b, &d, "span", vec![]);
    children(&mut b, &e, &[t, gap.clone()]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let id = c.subject_id(&e).unwrap();
    let r = report(&c, &id, 0);
    assert_eq!(
        value(&r),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(
            c.subject_id(&gap).unwrap()
        ))
    );
    assert_eq!(r.usage().nested_output_records(), 2);
    assert_eq!(r.usage().scratch_records(), 0);
    assert!(r.usage().allocated_bytes() >= 2 * std::mem::size_of::<u16>());
    // Missing can be a completed fact; an actual consulted storage denial is terminal instead.
    let mut s = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::NestedOutputRecords, 1),
        0,
        HtmlSemanticCancellation::new(),
    );
    let f = s.query(&[request(&id)]).unwrap_err();
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::NestedOutputRecords,
            ..
        }
    ));
    assert_eq!(s.usage().scratch_records(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), f);
}
