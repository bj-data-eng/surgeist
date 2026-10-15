use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedChildIds, HtmlCapturedIsValue, HtmlCapturedShadowProperties,
    HtmlSemanticCancellation, HtmlSemanticFact, HtmlSemanticFailureKind, HtmlSemanticLimits,
    HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport, HtmlSemanticRequest,
    HtmlSemanticResource, HtmlSemanticResult, HtmlSemanticSession, HtmlSemanticUnsupported,
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
use surgeist_html::*;

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
fn html(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    name: &str,
) -> HtmlCaptureBuilderSubject {
    expanded(
        b,
        d,
        QualifiedName::new(Namespace::html(), None, name),
        vec![],
    )
}
fn children(
    b: &mut HtmlCaptureBuilder,
    p: &HtmlCaptureBuilderSubject,
    list: &[HtmlCaptureBuilderSubject],
) {
    b.establish_children(p, list).unwrap();
}

fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20000000, 100, 1000, 0, 1000000, 0)
}

fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::AddressHasForbiddenDescendant)
}
fn report(c: &HtmlCapture, id: &HtmlCaptureSubjectId) -> HtmlSemanticReport {
    session(c, &context(c), limits(), 0, HtmlSemanticCancellation::new())
        .query(&[request(id)])
        .unwrap()
}
fn value(r: &HtmlSemanticReport) -> &HtmlSemanticValue<bool> {
    match r.results()[0].fact() {
        HtmlSemanticFact::AddressHasForbiddenDescendant(v) => v,
        other => panic!("expected address restriction, got {other:?}"),
    }
}
fn empty() -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let a = html(&mut b, &d, "address");
    children(&mut b, &a, &[]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    (c, id)
}
fn resource_limits(resource: HtmlSemanticResource, cap: usize) -> HtmlSemanticLimits {
    let l = limits();
    HtmlSemanticLimits::new(
        if resource == HtmlSemanticResource::Work {
            cap
        } else {
            l.work()
        },
        if resource == HtmlSemanticResource::OutputRecords {
            cap
        } else {
            l.output_records()
        },
        if resource == HtmlSemanticResource::ScratchRecords {
            cap
        } else {
            l.scratch_records()
        },
        0,
        if resource == HtmlSemanticResource::AllocatedBytes {
            cap
        } else {
            l.allocated_bytes()
        },
        0,
    )
}

#[test]
fn every_source_forbidden_heading_section_and_named_element_is_a_witness() {
    // Literal categories and named prohibitions from the selected HTML clauses.
    for name in [
        "h1", "h2", "h3", "h4", "h5", "h6", "article", "aside", "nav", "section", "header",
        "footer", "address",
    ] {
        let (mut b, d) = builder();
        let a = html(&mut b, &d, "address");
        let witness = html(&mut b, &d, name);
        // Witness children are deliberately unavailable; proof precedes descent.
        children(&mut b, &a, &[witness]);
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&a).unwrap())),
            &HtmlSemanticValue::Known(true),
            "{name}"
        );
    }
}

#[test]
fn empty_detached_address_excludes_itself_and_has_no_forbidden_descendant() {
    let (c, id) = empty();
    assert_eq!(value(&report(&c, &id)), &HtmlSemanticValue::Known(false));
}

#[test]
fn ordinary_nested_witness_is_found_through_neutral_wrappers() {
    let (mut b, d) = builder();
    let a = html(&mut b, &d, "address");
    let outer = html(&mut b, &d, "div");
    let inner = html(&mut b, &d, "span");
    let h = html(&mut b, &d, "h6");
    children(&mut b, &inner, &[h]);
    children(&mut b, &outer, &[inner]);
    children(&mut b, &a, &[outer]);
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&a).unwrap())),
        &HtmlSemanticValue::Known(true)
    );
}

#[test]
fn hgroup_without_heading_is_allowed_but_descendant_heading_is_forbidden() {
    for with_heading in [false, true] {
        let (mut b, d) = builder();
        let a = html(&mut b, &d, "address");
        let group = html(&mut b, &d, "hgroup");
        if with_heading {
            let div = html(&mut b, &d, "div");
            let h = html(&mut b, &d, "h2");
            children(&mut b, &div, &[h]);
            children(&mut b, &group, &[div]);
        } else {
            let p = html(&mut b, &d, "p");
            children(&mut b, &p, &[]);
            children(&mut b, &group, &[p]);
        }
        children(&mut b, &a, &[group]);
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&a).unwrap())),
            &HtmlSemanticValue::Known(with_heading)
        );
    }
}

#[test]
fn foreign_wrappers_are_traversed_but_foreign_forbidden_spellings_are_not_witnesses() {
    for with_html in [false, true] {
        let (mut b, d) = builder();
        let a = html(&mut b, &d, "address");
        let wrapper = expanded(
            &mut b,
            &d,
            QualifiedName::new(Namespace::svg(), None, "section"),
            vec![],
        );
        let h = expanded(
            &mut b,
            &d,
            QualifiedName::new(
                if with_html {
                    Namespace::html()
                } else {
                    Namespace::mathml()
                },
                None,
                "h1",
            ),
            vec![],
        );
        children(&mut b, &h, &[]);
        children(&mut b, &wrapper, &[h]);
        children(&mut b, &a, &[wrapper]);
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&a).unwrap())),
            &HtmlSemanticValue::Known(with_html)
        );
    }
}

#[test]
fn exact_expanded_names_ignore_prefix_but_preserve_case_and_utf16() {
    for (name, expected) in [
        (DocumentString::from("H1"), false),
        (DocumentString::from_utf16(vec![0x68, 0x31, 0xd800]), false),
        (DocumentString::from("h1"), true),
    ] {
        let (mut b, d) = builder();
        let a = expanded(
            &mut b,
            &d,
            QualifiedName::new(Namespace::html(), Some("a".into()), "address"),
            vec![],
        );
        let n = expanded(
            &mut b,
            &d,
            QualifiedName::new(Namespace::html(), Some("p".into()), name),
            vec![],
        );
        children(&mut b, &n, &[]);
        children(&mut b, &a, &[n]);
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&a).unwrap())),
            &HtmlSemanticValue::Known(expected)
        );
    }
}

#[test]
fn non_elements_wrong_roles_and_foreign_subjects_have_distinct_role_results() {
    let (mut b, d) = builder();
    let wrong = html(&mut b, &d, "div");
    let upper = html(&mut b, &d, "ADDRESS");
    let text = b.add_text(&d, "address".into()).unwrap();
    let f = b.add_fragment(&d).unwrap();
    let c = b.finish().unwrap();
    for subject in [&d, &wrong, &upper, &text, &f] {
        let r = report(&c, &c.subject_id(subject).unwrap());
        assert_eq!(value(&r), &HtmlSemanticValue::Inapplicable);
        assert_eq!(r.usage().peak_scratch_records(), 0);
    }
    for ns in [Namespace::none(), Namespace::svg(), Namespace::mathml()] {
        let (mut b, d) = builder();
        let a = expanded(&mut b, &d, QualifiedName::new(ns, None, "address"), vec![]);
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&a).unwrap())),
            &HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
        );
    }
}

#[test]
fn text_comments_pi_and_unconsulted_attribute_data_are_not_parsed() {
    let (mut b, d) = builder();
    let a = expanded(
        &mut b,
        &d,
        QualifiedName::new(Namespace::html(), None, "address"),
        vec![Attribute::new(
            QualifiedName::new(Namespace::none(), None, "title"),
            "x".repeat(10000),
        )],
    );
    let t = b
        .add_text(
            &d,
            DocumentString::from_utf16(vec![0xd800, 0, 0x20, 0x0d, 0x0a, 0xdc00]),
        )
        .unwrap();
    let comment = b.add_comment(&d, "<h1>".repeat(1000).into()).unwrap();
    let pi = b
        .add_processing_instruction(&d, "h1".into(), "<section>".into())
        .unwrap();
    children(&mut b, &a, &[t, comment, pi]);
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&a).unwrap());
    assert_eq!(value(&r), &HtmlSemanticValue::Known(false));
    assert_eq!(r.attribute_input_limit(), 0);
    assert_eq!(r.usage().nested_output_records(), 0);
}

#[test]
fn template_and_shadow_associations_are_excluded_but_ordinary_children_count() {
    for shadow in [false, true] {
        for ordinary_witness in [false, true] {
            let (mut b, d) = builder();
            let a = html(&mut b, &d, "address");
            let host = html(&mut b, &d, if shadow { "div" } else { "template" });
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
            let hidden = html(&mut b, &d, "header");
            children(&mut b, &f, &[hidden]);
            if ordinary_witness {
                let visible = html(&mut b, &d, "footer");
                children(&mut b, &host, &[visible]);
            } else {
                children(&mut b, &host, &[]);
            }
            children(&mut b, &a, &[host]);
            let c = b.finish().unwrap();
            assert_eq!(
                value(&report(&c, &c.subject_id(&a).unwrap())),
                &HtmlSemanticValue::Known(ordinary_witness)
            );
        }
    }
}

#[test]
fn unavailable_root_children_are_missing_not_false() {
    let (mut b, d) = builder();
    let a = html(&mut b, &d, "address");
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    let r = report(&c, &id);
    assert_eq!(
        value(&r),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(id))
    );
    assert_eq!(r.usage().scratch_records(), 0);
    assert_eq!(r.usage().peak_scratch_records(), 0);
}

#[test]
fn earlier_unavailable_branch_cannot_hide_later_independent_forbidden_witness() {
    for witness in [false, true] {
        let (mut b, d) = builder();
        let a = html(&mut b, &d, "address");
        let gap = html(&mut b, &d, "div");
        let later = html(&mut b, &d, if witness { "nav" } else { "p" });
        children(&mut b, &later, &[]);
        children(&mut b, &a, &[gap.clone(), later]);
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&a).unwrap());
        assert_eq!(
            value(&r),
            &if witness {
                HtmlSemanticValue::Known(true)
            } else {
                HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(
                    c.subject_id(&gap).unwrap(),
                ))
            }
        );
        assert_eq!(r.usage().scratch_records(), 0);
    }
}

#[test]
fn nested_gap_resumes_established_ancestor_order_to_a_later_witness() {
    let (mut b, d) = builder();
    let a = html(&mut b, &d, "address");
    let outer = html(&mut b, &d, "div");
    let gap = html(&mut b, &d, "span");
    children(&mut b, &outer, &[gap]);
    let later = html(&mut b, &d, "aside");
    children(&mut b, &a, &[outer, later]);
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&a).unwrap())),
        &HtmlSemanticValue::Known(true)
    );
}

#[test]
fn first_missing_dependency_is_retained_when_no_witness_resolves_it() {
    let (mut b, d) = builder();
    let a = html(&mut b, &d, "address");
    let first = html(&mut b, &d, "hgroup");
    let second = html(&mut b, &d, "div");
    children(&mut b, &a, &[first.clone(), second]);
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&a).unwrap())),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(
            c.subject_id(&first).unwrap()
        ))
    );
}

#[test]
fn witness_stops_before_later_unavailable_branch_and_huge_name() {
    let (mut b, d) = builder();
    let a = html(&mut b, &d, "address");
    let h = html(&mut b, &d, "h1");
    let later = html(&mut b, &d, "div");
    let huge = expanded(
        &mut b,
        &d,
        QualifiedName::new(Namespace::html(), None, "x".repeat(20000)),
        vec![],
    );
    children(&mut b, &a, &[h, later, huge]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    let r = report(&c, &id);
    assert_eq!(value(&r), &HtmlSemanticValue::Known(true));
    assert_eq!(r.usage().peak_scratch_records(), 1);
    let cx = context(&c);
    assert_eq!(
        value(
            &session(
                &c,
                &cx,
                resource_limits(HtmlSemanticResource::Work, r.usage().work()),
                0,
                HtmlSemanticCancellation::new()
            )
            .query(&[request(&id)])
            .unwrap()
        ),
        &HtmlSemanticValue::Known(true)
    );
}

#[test]
fn exact_and_one_short_work_scratch_bytes_and_batch_records_are_terminal() {
    let (c, id) = empty();
    let cx = context(&c);
    let r = report(&c, &id);
    let u = r.usage();
    // One vocabulary envelope, two namespace-unit passes, and eleven fixed
    // batch/node/iterator/reservation/result actions for this empty capture.
    assert_eq!(u.work(), 8 * KnownHtmlElement::ALL.len() + 2 * 28 + 11);
    assert_eq!(u.output_records(), 1);
    assert_eq!(u.peak_scratch_records(), 1);
    assert_eq!(u.nested_output_records(), 0);
    assert_eq!(
        u.allocated_bytes(),
        std::mem::size_of::<HtmlSemanticResult>() + std::mem::size_of::<HtmlCapturedChildIds<'_>>()
    );
    for (resource, exact) in [
        (HtmlSemanticResource::Work, u.work()),
        (HtmlSemanticResource::OutputRecords, 1),
        (HtmlSemanticResource::ScratchRecords, 1),
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
            value(&s.query(&[request(&id)]).unwrap()),
            &HtmlSemanticValue::Known(false)
        );
        let selected = resource_limits(resource, exact - 1);
        let mut short = session(&c, &cx, selected, 0, HtmlSemanticCancellation::new());
        let f = short.query(&[request(&id)]).unwrap_err();
        assert!(
            matches!(f.kind(), HtmlSemanticFailureKind::LimitExceeded {resource:r,..} if *r==resource)
        );
        assert_eq!(f.qualification().unwrap().limits(), selected);
        assert_eq!(short.usage().scratch_records(), 0);
        assert_eq!(short.query(&[]).unwrap_err(), f);
        assert_eq!(value(&r), &HtmlSemanticValue::Known(false));
    }
}

#[test]
fn terminal_failure_before_later_witness_is_not_overridden_by_existential_proof() {
    let (mut b, d) = builder();
    let a = html(&mut b, &d, "address");
    let gap = html(&mut b, &d, "div");
    let expensive = html(&mut b, &d, &"x".repeat(20000));
    let witness = html(&mut b, &d, "h1");
    children(&mut b, &a, &[gap, expensive, witness]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    let cx = context(&c);
    let measured = report(&c, &id);
    assert_eq!(value(&measured), &HtmlSemanticValue::Known(true));
    let mut s = session(
        &c,
        &cx,
        // The consulted 20,000-unit name exceeds this work envelope before
        // reaching the later heading; unknown earlier topology cannot mask it.
        resource_limits(HtmlSemanticResource::Work, 10000),
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
    assert_eq!(s.usage().scratch_records(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), f);
}

#[test]
fn scratch_refusal_after_unknown_gap_precedes_nested_witness_and_releases_root_frame() {
    let (mut b, d) = builder();
    let a = html(&mut b, &d, "address");
    let gap = html(&mut b, &d, "div");
    let wrapper = html(&mut b, &d, "span");
    let witness = html(&mut b, &d, "h1");
    children(&mut b, &wrapper, &[witness]);
    children(&mut b, &a, &[gap, wrapper]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    let cx = context(&c);
    assert_eq!(value(&report(&c, &id)), &HtmlSemanticValue::Known(true));
    let mut s = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::ScratchRecords, 1),
        0,
        HtmlSemanticCancellation::new(),
    );
    let f = s.query(&[request(&id)]).unwrap_err();
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::ScratchRecords,
            limit: 1,
            required: 2
        }
    ));
    assert_eq!(s.usage().scratch_records(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), f);
}

#[test]
fn repeated_and_duplicate_queries_admit_real_cumulative_scalar_and_scratch_storage() {
    let (c, id) = empty();
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let one = s.query(&[request(&id)]).unwrap();
    let two = s.query(&[request(&id)]).unwrap();
    assert_eq!(two.usage().work(), 2 * one.usage().work());
    assert_eq!(
        two.usage().allocated_bytes(),
        2 * one.usage().allocated_bytes()
    );
    assert_eq!(two.usage().output_records(), 2);
    assert_eq!(two.usage().scratch_records(), 0);
    let mut s = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let batch = s.query(&[request(&id), request(&id)]).unwrap();
    assert_eq!(batch.results().len(), 2);
    assert_eq!(batch.usage().output_records(), 2);
    assert_eq!(
        batch.usage().allocated_bytes(),
        2 * one.usage().allocated_bytes()
    );
    assert_eq!(batch.usage().nested_output_records(), 0);
}

#[test]
fn exact_batch_work_and_one_short_late_failure_discard_prefix_and_release_frames() {
    let (c, id) = empty();
    let cx = context(&c);
    let mut measure = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let measured = measure.query(&[request(&id), request(&id)]).unwrap();
    let exact = measured.usage().work();
    let mut s = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::Work, exact),
        0,
        HtmlSemanticCancellation::new(),
    );
    assert_eq!(
        s.query(&[request(&id), request(&id)])
            .unwrap()
            .results()
            .len(),
        2
    );
    let mut short = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::Work, exact - 1),
        0,
        HtmlSemanticCancellation::new(),
    );
    let f = short.query(&[request(&id), request(&id)]).unwrap_err();
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            ..
        }
    ));
    assert_eq!(short.usage().output_records(), 2);
    assert_eq!(short.usage().scratch_records(), 0);
    assert_eq!(short.query(&[]).unwrap_err(), f);
    assert_eq!(value(&measured), &HtmlSemanticValue::Known(false));
}

#[test]
fn cancellation_before_query_and_after_success_retains_first_stop_without_more_work() {
    let (c, id) = empty();
    let cx = context(&c);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut s = session(&c, &cx, limits(), 0, cancel);
    let f = s.query(&[request(&id)]).unwrap_err();
    assert_eq!(f.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(s.usage().work(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), f);
    let cancel = HtmlSemanticCancellation::new();
    let mut s = session(&c, &cx, limits(), 0, cancel.clone());
    let old = s.query(&[request(&id)]).unwrap();
    cancel.cancel();
    let f = s.query(&[request(&id)]).unwrap_err();
    assert_eq!(f.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(f.qualification().unwrap(), old.qualification());
    assert_eq!(s.usage().work(), old.usage().work());
    assert_eq!(s.query(&[]).unwrap_err(), f);
    assert_eq!(value(&old), &HtmlSemanticValue::Known(false));
}

#[test]
fn capture_context_host_configuration_limits_and_foreign_keys_are_qualified() {
    let (c, id) = empty();
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 7, HtmlSemanticCancellation::new());
    let r = s.query(&[request(&id)]).unwrap();
    assert_eq!(r.qualification().capture(), c.identity());
    assert_eq!(r.qualification().context(), cx.identity());
    assert_eq!(r.qualification().host_revision(), c.host_facts_revision());
    assert_eq!(r.qualification().limits(), limits());
    assert_eq!(r.attribute_input_limit(), 7);
    let other_cx = context(&c);
    let changed = session(&c, &other_cx, limits(), 8, HtmlSemanticCancellation::new())
        .query(&[request(&id)])
        .unwrap();
    assert_eq!(value(&changed), &HtmlSemanticValue::Known(false));
    assert_ne!(changed.qualification(), r.qualification());
    let (other, foreign) = empty();
    let f = s.query(&[request(&id), request(&foreign)]).unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Capture(_)));
    assert_eq!(s.usage().scratch_records(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), f);
    assert!(
        HtmlSemanticSession::new(
            &other,
            &cx,
            HtmlValueInterpreter::new(7),
            limits(),
            HtmlSemanticCancellation::new()
        )
        .is_err()
    );
    assert_eq!(value(&r), &HtmlSemanticValue::Known(false));
}

#[test]
fn changed_capture_topology_recomputes_and_prior_report_remains_valid() {
    let mut reports = Vec::new();
    for with_heading in [false, true] {
        let (mut b, d) = builder();
        let a = html(&mut b, &d, "address");
        let child = html(&mut b, &d, if with_heading { "h3" } else { "p" });
        children(&mut b, &child, &[]);
        children(&mut b, &a, &[child]);
        let c = b.finish().unwrap();
        reports.push(report(&c, &c.subject_id(&a).unwrap()));
    }
    assert_eq!(value(&reports[0]), &HtmlSemanticValue::Known(false));
    assert_eq!(value(&reports[1]), &HtmlSemanticValue::Known(true));
    assert_ne!(
        reports[0].qualification().capture(),
        reports[1].qualification().capture()
    );
    assert_ne!(
        reports[0].qualification().host_revision(),
        reports[1].qualification().host_revision()
    );
}
