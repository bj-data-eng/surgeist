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
fn text(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    units: impl Into<DocumentString>,
) -> HtmlCaptureBuilderSubject {
    b.add_text(d, units.into()).unwrap()
}
fn children(
    b: &mut HtmlCaptureBuilder,
    p: &HtmlCaptureBuilderSubject,
    list: &[HtmlCaptureBuilderSubject],
) {
    b.establish_children(p, list).unwrap();
}

fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20000000, 100, 1000, 0, 1000000, 100000)
}

fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::AnchorText)
}
fn report(c: &HtmlCapture, id: &HtmlCaptureSubjectId) -> HtmlSemanticReport {
    session(c, &context(c), limits(), 0, HtmlSemanticCancellation::new())
        .query(&[request(id)])
        .unwrap()
}
fn value(r: &HtmlSemanticReport) -> &HtmlSemanticValue<DocumentString> {
    match r.results()[0].fact() {
        HtmlSemanticFact::AnchorText(v) => v,
        other => panic!("expected anchor text, got {other:?}"),
    }
}
fn known(r: &HtmlSemanticReport) -> &DocumentString {
    match value(r) {
        HtmlSemanticValue::Known(v) => v,
        other => panic!("expected exact text, got {other:?}"),
    }
}
fn leaf(units: impl Into<DocumentString>) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let a = html(&mut b, &d, "a");
    let t = text(&mut b, &d, units);
    children(&mut b, &a, &[t]);
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
        if resource == HtmlSemanticResource::NestedOutputRecords {
            cap
        } else {
            l.nested_output_records()
        },
    )
}

#[test]
fn descendant_text_includes_scripts_images_and_foreign_wrappers_in_ordinary_order() {
    let (mut b, d) = builder();
    let a = html(&mut b, &d, "a");
    let direct = text(&mut b, &d, "A");
    let script = html(&mut b, &d, "script");
    let script_text = text(&mut b, &d, "B");
    children(&mut b, &script, &[script_text]);
    let svg = expanded(
        &mut b,
        &d,
        QualifiedName::new(Namespace::svg(), None, "script"),
        vec![],
    );
    let svg_text = text(&mut b, &d, "C");
    children(&mut b, &svg, &[svg_text]);
    let img = expanded(
        &mut b,
        &d,
        QualifiedName::new(Namespace::html(), None, "img"),
        vec![attr("alt", "unused")],
    );
    let img_text = text(&mut b, &d, "D");
    children(&mut b, &img, &[img_text]);
    let br = html(&mut b, &d, "br");
    children(&mut b, &br, &[]);
    let tail = text(&mut b, &d, "E");
    let comment = b.add_comment(&d, "ignored".into()).unwrap();
    let pi = b
        .add_processing_instruction(&d, "x".into(), "ignored".into())
        .unwrap();
    children(
        &mut b,
        &a,
        &[direct, script, comment, svg, br, img, pi, tail],
    );
    let c = b.finish().unwrap();
    assert_eq!(
        known(&report(&c, &c.subject_id(&a).unwrap())).code_units(),
        [65, 66, 67, 68, 69]
    );
}

#[test]
fn literal_whitespace_newlines_null_and_surrogates_are_not_normalized() {
    let expected = [
        0x20, 0x09, 0x0d, 0x0a, 0x0c, 0, 0xd800, 0x41, 0xdc00, 0xd83d, 0xde42, 0x20,
    ];
    let (c, id) = leaf(DocumentString::from_utf16(expected.to_vec()));
    assert_eq!(known(&report(&c, &id)).code_units(), expected);
}

#[test]
fn href_title_and_ancestry_are_unconsulted_even_at_zero_attribute_input_cap() {
    for with_href in [false, true] {
        let (mut b, d) = builder();
        let mut attrs = vec![attr("title", "x".repeat(10000))];
        if with_href {
            attrs.push(attr("href", "y".repeat(10000)));
        }
        let a = expanded(
            &mut b,
            &d,
            QualifiedName::new(Namespace::html(), None, "a"),
            attrs,
        );
        let t = text(&mut b, &d, "exact");
        children(&mut b, &a, &[t]);
        let c = b.finish().unwrap();
        assert_eq!(
            known(&report(&c, &c.subject_id(&a).unwrap())).code_units(),
            [101, 120, 97, 99, 116]
        );
    }
}

#[test]
fn complete_empty_and_nontext_children_establish_known_empty() {
    for with_comment in [false, true] {
        let (mut b, d) = builder();
        let a = html(&mut b, &d, "a");
        if with_comment {
            let comment = b.add_comment(&d, "x".repeat(10000).into()).unwrap();
            let pi = b
                .add_processing_instruction(&d, "x".into(), "y".repeat(10000).into())
                .unwrap();
            children(&mut b, &a, &[comment, pi]);
        } else {
            children(&mut b, &a, &[]);
        }
        b.establish_root(&a).unwrap();
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&a).unwrap());
        assert!(known(&r).is_empty());
        assert_eq!(r.usage().nested_output_records(), 0);
        assert_eq!(r.usage().scratch_records(), 0);
        if !with_comment {
            assert_eq!(r.usage().peak_scratch_records(), 0);
        }
    }
}

#[test]
fn root_and_later_descendant_unknown_children_remain_missing_not_partial_text() {
    let (mut b, d) = builder();
    let a = html(&mut b, &d, "a");
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    let r = report(&c, &id);
    assert_eq!(
        value(&r),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(id))
    );
    assert_eq!(r.usage().nested_output_records(), 0);
    let (mut b, d) = builder();
    let a = html(&mut b, &d, "a");
    let prefix = text(&mut b, &d, "AB");
    let gap = html(&mut b, &d, "script");
    children(&mut b, &a, &[prefix, gap.clone()]);
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&a).unwrap());
    assert_eq!(
        value(&r),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(
            c.subject_id(&gap).unwrap()
        ))
    );
    assert_eq!(r.usage().nested_output_records(), 2);
    assert!(r.usage().allocated_bytes() > std::mem::size_of::<HtmlSemanticResult>());
    assert_eq!(r.usage().scratch_records(), 0);
}

#[test]
fn namespace_case_and_non_element_roles_do_not_guess_an_anchor() {
    let (mut b, d) = builder();
    let a = html(&mut b, &d, "A");
    let area = html(&mut b, &d, "area");
    let custom = html(&mut b, &d, "x-a");
    let isolated = expanded(
        &mut b,
        &d,
        QualifiedName::new(
            Namespace::html(),
            None,
            DocumentString::from_utf16(vec![0x61, 0xd800]),
        ),
        vec![],
    );
    let text = text(&mut b, &d, "a");
    let fragment = b.add_fragment(&d).unwrap();
    let c = b.finish().unwrap();
    for subject in [&d, &a, &area, &custom, &isolated, &text, &fragment] {
        let r = report(&c, &c.subject_id(subject).unwrap());
        assert_eq!(value(&r), &HtmlSemanticValue::Inapplicable);
        assert_eq!(r.usage().peak_scratch_records(), 0);
        assert_eq!(r.usage().nested_output_records(), 0);
    }
    for ns in [Namespace::none(), Namespace::svg(), Namespace::mathml()] {
        let (mut b, d) = builder();
        let a = expanded(&mut b, &d, QualifiedName::new(ns, None, "a"), vec![]);
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&a).unwrap())),
            &HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
        );
    }
}

#[test]
fn html_prefix_is_not_part_of_expanded_anchor_identity() {
    let (mut b, d) = builder();
    let a = expanded(
        &mut b,
        &d,
        QualifiedName::new(Namespace::html(), Some("p".into()), "a"),
        vec![],
    );
    let t = text(&mut b, &d, "prefix-independent");
    children(&mut b, &a, &[t]);
    let c = b.finish().unwrap();
    assert_eq!(
        known(&report(&c, &c.subject_id(&a).unwrap()))
            .to_utf8()
            .unwrap(),
        "prefix-independent"
    );
}

#[test]
fn template_and_shadow_associations_are_excluded_but_real_ordinary_children_remain() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let a = html(&mut b, &d, "a");
        let host = html(&mut b, &d, if shadow { "div" } else { "template" });
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
        let excluded = text(&mut b, &d, "associated");
        children(&mut b, &f, &[excluded]);
        let ordinary = text(&mut b, &d, "ordinary");
        children(&mut b, &host, &[ordinary]);
        children(&mut b, &a, &[host]);
        let c = b.finish().unwrap();
        assert_eq!(
            known(&report(&c, &c.subject_id(&a).unwrap()))
                .to_utf8()
                .unwrap(),
            "ordinary"
        );
    }
}

#[test]
fn exact_and_one_short_work_scratch_nested_output_bytes_and_batch_storage_are_terminal() {
    let (c, id) = leaf("AB");
    let cx = context(&c);
    let r = report(&c, &id);
    let u = r.usage();
    assert_eq!(known(&r).code_units(), [65, 66]);
    assert_eq!(u.output_records(), 1);
    assert_eq!(u.nested_output_records(), 2);
    assert_eq!(u.peak_scratch_records(), 1);
    assert_eq!(
        u.allocated_bytes(),
        std::mem::size_of::<HtmlSemanticResult>()
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
        let mut exact_session = session(
            &c,
            &cx,
            resource_limits(resource, exact),
            0,
            HtmlSemanticCancellation::new(),
        );
        assert_eq!(
            known(&exact_session.query(&[request(&id)]).unwrap()).code_units(),
            [65, 66]
        );
        let selected = resource_limits(resource, exact - 1);
        let mut short = session(&c, &cx, selected, 0, HtmlSemanticCancellation::new());
        let failure = short.query(&[request(&id)]).unwrap_err();
        assert!(
            matches!(failure.kind(), HtmlSemanticFailureKind::LimitExceeded {resource:r,..} if *r==resource)
        );
        assert_eq!(short.usage().scratch_records(), 0);
        assert_eq!(failure.qualification().unwrap().limits(), selected);
        assert_eq!(short.query(&[]).unwrap_err(), failure);
        assert_eq!(known(&r).code_units(), [65, 66]);
    }
}

#[test]
fn repeated_queries_and_duplicate_requests_admit_cumulative_real_materialization() {
    let (c, id) = leaf("AB");
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let req = request(&id);
    let first = s.query(std::slice::from_ref(&req)).unwrap();
    let second = s.query(std::slice::from_ref(&req)).unwrap();
    assert_eq!(second.usage().work(), 2 * first.usage().work());
    assert_eq!(
        second.usage().allocated_bytes(),
        2 * first.usage().allocated_bytes()
    );
    assert_eq!(
        second.usage().output_records(),
        2 * first.usage().output_records()
    );
    assert_eq!(
        second.usage().nested_output_records(),
        2 * first.usage().nested_output_records()
    );
    assert_eq!(second.usage().scratch_records(), 0);
    let mut s = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let batch = s.query(&[request(&id), request(&id)]).unwrap();
    assert_eq!(batch.results().len(), 2);
    assert_eq!(batch.usage().nested_output_records(), 4);
    assert_eq!(known(&first).code_units(), [65, 66]);
}

#[test]
fn late_text_limit_discards_the_batch_prefix_and_releases_all_live_frames() {
    let (mut b, d) = builder();
    let a = html(&mut b, &d, "a");
    let t = text(&mut b, &d, "AB");
    children(&mut b, &a, &[t]);
    let late = html(&mut b, &d, "a");
    let large = text(&mut b, &d, "CD");
    children(&mut b, &late, &[large]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    let cx = context(&c);
    let mut s = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::NestedOutputRecords, 3),
        0,
        HtmlSemanticCancellation::new(),
    );
    let prior = report(&c, &id);
    let failure = s
        .query(&[request(&id), request(&c.subject_id(&late).unwrap())])
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::NestedOutputRecords,
            ..
        }
    ));
    assert_eq!(s.usage().nested_output_records(), 3);
    assert_eq!(s.usage().output_records(), 2);
    assert_eq!(s.usage().scratch_records(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), failure);
    assert_eq!(known(&prior).code_units(), [65, 66]);
}

#[test]
fn exact_batch_work_and_one_short_late_failure_keep_qualification_and_prior_reports() {
    let (c, id) = leaf("AB");
    let cx = context(&c);
    let mut measuring = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let measured = measuring
        .query(&[request(&id), request(&id)])
        .unwrap()
        .usage()
        .work();
    let mut exact = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::Work, measured),
        0,
        HtmlSemanticCancellation::new(),
    );
    assert_eq!(
        exact
            .query(&[request(&id), request(&id)])
            .unwrap()
            .results()
            .len(),
        2
    );
    let mut short = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::Work, measured - 1),
        0,
        HtmlSemanticCancellation::new(),
    );
    let failure = short.query(&[request(&id), request(&id)]).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            ..
        }
    ));
    assert!(short.usage().nested_output_records() >= 2);
    assert_eq!(short.usage().scratch_records(), 0);
    assert_eq!(short.query(&[]).unwrap_err(), failure);
    assert_eq!(known(&report(&c, &id)).code_units(), [65, 66]);
}

#[test]
fn cancellation_before_query_and_after_success_is_sticky_without_additional_work() {
    let (c, id) = leaf("text");
    let cx = context(&c);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut s = session(&c, &cx, limits(), 0, cancel);
    let failure = s.query(&[request(&id)]).unwrap_err();
    assert_eq!(failure.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(s.usage().work(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), failure);
    let cancel = HtmlSemanticCancellation::new();
    let mut s = session(&c, &cx, limits(), 0, cancel.clone());
    let earlier = s.query(&[request(&id)]).unwrap();
    cancel.cancel();
    let failure = s.query(&[request(&id)]).unwrap_err();
    assert_eq!(failure.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(failure.qualification().unwrap(), earlier.qualification());
    assert_eq!(s.usage().work(), earlier.usage().work());
    assert_eq!(s.query(&[]).unwrap_err(), failure);
    assert_eq!(known(&earlier).to_utf8().unwrap(), "text");
}

#[test]
fn capture_context_host_limits_raw_configuration_and_foreign_keys_remain_exact() {
    let (c, id) = leaf("old");
    let cx = context(&c);
    let alternate = context(&c);
    let mut s = session(&c, &cx, limits(), 7, HtmlSemanticCancellation::new());
    let r = s.query(&[request(&id)]).unwrap();
    assert_eq!(r.qualification().capture(), c.identity());
    assert_eq!(r.qualification().context(), cx.identity());
    assert_eq!(r.qualification().host_revision(), c.host_facts_revision());
    assert_eq!(r.qualification().limits(), limits());
    assert_eq!(r.qualification().attribute_input_limit(), 7);
    assert_ne!(cx.identity(), alternate.identity());
    let alternate_limits = HtmlSemanticLimits::new(20000001, 100, 1000, 0, 1000000, 100000);
    let changed = session(
        &c,
        &alternate,
        alternate_limits,
        8,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(&id)])
    .unwrap();
    assert_eq!(known(&changed).to_utf8().unwrap(), "old");
    assert_eq!(changed.context(), alternate.identity());
    assert_ne!(changed.qualification(), r.qualification());
    assert_eq!(changed.limits(), alternate_limits);
    assert_eq!(changed.attribute_input_limit(), 8);
    let (other, foreign) = leaf("new");
    let failure = s.query(&[request(&id), request(&foreign)]).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Capture(_)
    ));
    assert_eq!(failure.qualification().unwrap(), r.qualification());
    assert_eq!(s.query(&[]).unwrap_err(), failure);
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
    assert_eq!(known(&r).to_utf8().unwrap(), "old");
}

#[test]
fn changed_child_order_and_text_recompute_and_old_backing_survives() {
    let mut reports = Vec::new();
    for reversed in [false, true] {
        let (mut b, d) = builder();
        let a = html(&mut b, &d, "a");
        let left = text(&mut b, &d, "A");
        let right = text(&mut b, &d, if reversed { "C" } else { "B" });
        if reversed {
            children(&mut b, &a, &[right, left]);
        } else {
            children(&mut b, &a, &[left, right]);
        }
        let c = b.finish().unwrap();
        reports.push(report(&c, &c.subject_id(&a).unwrap()));
    }
    assert_eq!(known(&reports[0]).to_utf8().unwrap(), "AB");
    assert_eq!(known(&reports[1]).to_utf8().unwrap(), "CA");
    assert_ne!(reports[0].capture(), reports[1].capture());
    assert_ne!(reports[0].host_revision(), reports[1].host_revision());
    assert_ne!(reports[0].context(), reports[1].context());
    assert_eq!(
        reports[0].dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
}

#[test]
fn deeper_ordinary_descendants_admit_and_release_actual_frame_capacity() {
    let (mut b, d) = builder();
    let a = html(&mut b, &d, "a");
    let span = html(&mut b, &d, "span");
    let em = html(&mut b, &d, "em");
    let t = text(&mut b, &d, "deep");
    children(&mut b, &em, &[t]);
    children(&mut b, &span, &[em]);
    children(&mut b, &a, &[span]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    let cx = context(&c);
    let r = report(&c, &id);
    assert_eq!(known(&r).to_utf8().unwrap(), "deep");
    assert_eq!(r.usage().peak_scratch_records(), 3);
    assert_eq!(r.usage().scratch_records(), 0);
    let mut exact = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::ScratchRecords, 3),
        0,
        HtmlSemanticCancellation::new(),
    );
    assert_eq!(
        known(&exact.query(&[request(&id)]).unwrap())
            .to_utf8()
            .unwrap(),
        "deep"
    );
    let mut short = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::ScratchRecords, 2),
        0,
        HtmlSemanticCancellation::new(),
    );
    let failure = short.query(&[request(&id)]).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::ScratchRecords,
            ..
        }
    ));
    assert_eq!(short.usage().scratch_records(), 0);
    assert_eq!(short.query(&[]).unwrap_err(), failure);
}

#[test]
fn nonterminal_missing_retains_no_text_prefix_and_allows_later_complete_anchor() {
    let (mut b, d) = builder();
    let missing = html(&mut b, &d, "a");
    let complete = html(&mut b, &d, "a");
    let t = text(&mut b, &d, "later");
    children(&mut b, &complete, &[t]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let missing = c.subject_id(&missing).unwrap();
    let mut s = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let r = s
        .query(&[
            request(&missing),
            request(&c.subject_id(&complete).unwrap()),
        ])
        .unwrap();
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::AnchorText(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::OrderedChildren(missing)
        ))
    );
    assert_eq!(
        r.results()[1].fact(),
        &HtmlSemanticFact::AnchorText(HtmlSemanticValue::Known("later".into()))
    );
    assert_eq!(r.usage().scratch_records(), 0);
}
