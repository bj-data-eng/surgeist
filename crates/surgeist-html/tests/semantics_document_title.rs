#[path = "common/context.rs"]
mod shared_context;
use shared_context::context;
use surgeist_html::*;

fn builder() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(1000, 100000, 1000, 1000000, 1000, 0, 0),
        HtmlHostFactsRevision::fresh(),
    );
    let d = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    (b, d)
}
fn expanded(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    ns: Namespace,
    prefix: Option<&str>,
    local: &str,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(ns, prefix.map(DocumentString::from), local),
        OrderedAttributes::default(),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn html(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    local: &str,
) -> HtmlCaptureBuilderSubject {
    expanded(b, d, Namespace::html(), None, local)
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
    HtmlSemanticLimits::new(1000000, 100, 1000, 0, 1000000, 10000)
}
fn session<'a>(
    c: &'a HtmlCapture,
    cx: &'a HtmlSemanticContext,
    l: HtmlSemanticLimits,
    cancel: HtmlSemanticCancellation,
) -> HtmlSemanticSession<'a> {
    // Text getters do not consume raw attributes; zero is intentional.
    HtmlSemanticSession::new(c, cx, HtmlValueInterpreter::new(0), l, cancel).unwrap()
}
fn req(id: &HtmlCaptureSubjectId, q: HtmlSemanticQuery) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), q)
}
fn report(c: &HtmlCapture, id: &HtmlCaptureSubjectId, q: HtmlSemanticQuery) -> HtmlSemanticReport {
    session(c, &context(c), limits(), HtmlSemanticCancellation::new())
        .query(&[req(id, q)])
        .unwrap()
}
fn doc_value(r: &HtmlSemanticReport) -> &HtmlSemanticValue<HtmlDocumentTitle> {
    match r.results()[0].fact() {
        HtmlSemanticFact::DocumentTitle(v) => v,
        other => panic!("{other:?}"),
    }
}
fn doc_known(r: &HtmlSemanticReport) -> &HtmlDocumentTitle {
    match doc_value(r) {
        HtmlSemanticValue::Known(v) => v,
        other => panic!("{other:?}"),
    }
}
fn title_value(r: &HtmlSemanticReport) -> &HtmlSemanticValue<DocumentString> {
    match r.results()[0].fact() {
        HtmlSemanticFact::TitleText(v) => v,
        other => panic!("{other:?}"),
    }
}
fn single(raw: DocumentString) -> (HtmlCapture, HtmlCaptureSubjectId, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let t = html(&mut b, &d, "title");
    let a = text(&mut b, &d, raw);
    children(&mut b, &t, &[a]);
    children(&mut b, &d, std::slice::from_ref(&t));
    let c = b.finish().unwrap();
    let di = c.subject_id(&d).unwrap();
    let ti = c.subject_id(&t).unwrap();
    (c, di, ti)
}

#[test]
fn direct_text_is_exact_but_document_title_collapses_only_ascii_whitespace() {
    let (mut b, d) = builder();
    let t = html(&mut b, &d, "title");
    let a = text(&mut b, &d, DocumentString::from_utf16(vec![32, 9, 65, 13]));
    let ignored = html(&mut b, &d, "span"); // Its unknown children are irrelevant.
    let comment = b.add_comment(&d, "not title".into()).unwrap();
    let pi = b
        .add_processing_instruction(&d, "x".into(), "ignored".into())
        .unwrap();
    let z = text(
        &mut b,
        &d,
        DocumentString::from_utf16(vec![
            10, 12, 32, 66, 11, 160, 0, 0xd83d, 0xde00, 0xd800, 67, 0xdc00, 32,
        ]),
    );
    children(&mut b, &t, &[a, ignored, comment, pi, z]);
    children(&mut b, &d, std::slice::from_ref(&t));
    let c = b.finish().unwrap();
    let ti = c.subject_id(&t).unwrap();
    let di = c.subject_id(&d).unwrap();
    let r = report(&c, &ti, HtmlSemanticQuery::TitleText);
    assert_eq!(
        title_value(&r),
        &HtmlSemanticValue::Known(DocumentString::from_utf16(vec![
            32, 9, 65, 13, 10, 12, 32, 66, 11, 160, 0, 0xd83d, 0xde00, 0xd800, 67, 0xdc00, 32
        ]))
    );
    let r = report(&c, &di, HtmlSemanticQuery::DocumentTitle);
    assert_eq!(
        doc_known(&r).value().code_units(),
        [65, 32, 66, 11, 160, 0, 0xd83d, 0xde00, 0xd800, 67, 0xdc00]
    );
    assert_eq!(doc_known(&r).title_element(), Some(&ti));
    assert_eq!(r.usage().scratch_records(), 0);
}

#[test]
fn first_html_title_outside_head_wins_and_later_unknown_subtrees_are_irrelevant() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "body");
    let first = html(&mut b, &d, "title");
    let later = html(&mut b, &d, "section");
    children(&mut b, &first, &[]);
    children(&mut b, &root, &[first.clone(), later]);
    children(&mut b, &d, &[root]);
    let c = b.finish().unwrap();
    let r = report(
        &c,
        &c.subject_id(&d).unwrap(),
        HtmlSemanticQuery::DocumentTitle,
    );
    assert!(doc_known(&r).value().is_empty());
    assert_eq!(
        doc_known(&r).title_element(),
        Some(&c.subject_id(&first).unwrap())
    );
}

#[test]
fn earlier_unknown_subtree_blocks_a_later_known_title() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "html");
    let earlier = html(&mut b, &d, "section");
    let t = html(&mut b, &d, "title");
    children(&mut b, &t, &[]);
    children(&mut b, &root, &[earlier.clone(), t]);
    children(&mut b, &d, &[root]);
    let c = b.finish().unwrap();
    let r = report(
        &c,
        &c.subject_id(&d).unwrap(),
        HtmlSemanticQuery::DocumentTitle,
    );
    assert_eq!(
        doc_value(&r),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(
            c.subject_id(&earlier).unwrap()
        ))
    );
    assert_eq!(r.usage().scratch_records(), 0);
    assert!(r.usage().peak_scratch_records() > 0);
}

#[test]
fn first_title_unknown_children_are_not_skipped() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "html");
    let t = html(&mut b, &d, "title");
    let other = html(&mut b, &d, "title");
    children(&mut b, &other, &[]);
    children(&mut b, &root, &[t.clone(), other]);
    children(&mut b, &d, &[root]);
    let c = b.finish().unwrap();
    for (id, q) in [
        (c.subject_id(&d).unwrap(), HtmlSemanticQuery::DocumentTitle),
        (c.subject_id(&t).unwrap(), HtmlSemanticQuery::TitleText),
    ] {
        let r = report(&c, &id, q);
        let expected = HtmlSemanticMissing::OrderedChildren(c.subject_id(&t).unwrap());
        match q {
            HtmlSemanticQuery::DocumentTitle => {
                assert_eq!(doc_value(&r), &HtmlSemanticValue::Missing(expected))
            }
            _ => assert_eq!(title_value(&r), &HtmlSemanticValue::Missing(expected)),
        }
        assert_eq!(r.usage().scratch_records(), 0);
    }
}

#[test]
fn document_children_must_be_known_before_root_selection() {
    let (b, d) = builder();
    let c = b.finish().unwrap();
    let r = report(
        &c,
        &c.subject_id(&d).unwrap(),
        HtmlSemanticQuery::DocumentTitle,
    );
    assert_eq!(
        doc_value(&r),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(
            c.subject_id(&d).unwrap()
        ))
    );
}

#[test]
fn ambiguous_document_element_is_local_unsupported_without_capture_rejection() {
    let (mut b, d) = builder();
    let a = html(&mut b, &d, "title");
    let z = expanded(&mut b, &d, Namespace::svg(), None, "svg");
    children(&mut b, &d, &[a, z]);
    let c = b.finish().unwrap();
    let r = report(
        &c,
        &c.subject_id(&d).unwrap(),
        HtmlSemanticQuery::DocumentTitle,
    );
    assert_eq!(
        doc_value(&r),
        &HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::AmbiguousDocumentElement)
    );
}

#[test]
fn no_document_element_and_complete_no_title_have_known_empty_without_source() {
    for with_root in [false, true] {
        let (mut b, d) = builder();
        let mut list = vec![];
        if with_root {
            let root = html(&mut b, &d, "html");
            children(&mut b, &root, &[]);
            list.push(root);
        }
        children(&mut b, &d, &list);
        let c = b.finish().unwrap();
        let r = report(
            &c,
            &c.subject_id(&d).unwrap(),
            HtmlSemanticQuery::DocumentTitle,
        );
        assert!(doc_known(&r).value().is_empty());
        assert_eq!(doc_known(&r).title_element(), None);
        assert_eq!(r.usage().nested_output_records(), 0);
    }
}

#[test]
fn mixed_foreign_wrappers_allow_html_title_and_prefix_does_not_change_identity() {
    let (mut b, d) = builder();
    let root = expanded(&mut b, &d, Namespace::mathml(), None, "math");
    let t = expanded(&mut b, &d, Namespace::html(), Some("p"), "title");
    let a = text(&mut b, &d, " selected ");
    children(&mut b, &t, &[a]);
    children(&mut b, &root, std::slice::from_ref(&t));
    children(&mut b, &d, &[root]);
    let c = b.finish().unwrap();
    let r = report(
        &c,
        &c.subject_id(&d).unwrap(),
        HtmlSemanticQuery::DocumentTitle,
    );
    assert_eq!(
        doc_known(&r).value().code_units(),
        "selected".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(
        doc_known(&r).title_element(),
        Some(&c.subject_id(&t).unwrap())
    );
}

#[test]
fn svg_root_selects_first_direct_svg_title_not_nested_or_html_title() {
    let (mut b, d) = builder();
    let root = expanded(&mut b, &d, Namespace::svg(), Some("s"), "svg");
    let nested = expanded(&mut b, &d, Namespace::svg(), None, "g");
    let nt = expanded(&mut b, &d, Namespace::svg(), None, "title");
    let html_title = html(&mut b, &d, "title");
    let selected = expanded(&mut b, &d, Namespace::svg(), Some("s"), "title");
    let later = expanded(&mut b, &d, Namespace::svg(), None, "title");
    let a = text(&mut b, &d, " SVG \t title ");
    children(&mut b, &nested, &[nt]);
    children(&mut b, &selected, &[a]);
    children(
        &mut b,
        &root,
        &[nested, html_title, selected.clone(), later],
    );
    children(&mut b, &d, &[root]);
    let c = b.finish().unwrap();
    let r = report(
        &c,
        &c.subject_id(&d).unwrap(),
        HtmlSemanticQuery::DocumentTitle,
    );
    assert_eq!(
        doc_known(&r).value().code_units(),
        "SVG title".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(
        doc_known(&r).title_element(),
        Some(&c.subject_id(&selected).unwrap())
    );
    assert_eq!(r.usage().peak_scratch_records(), 0);
}

#[test]
fn svg_absence_never_falls_back_and_empty_selection_retains_source() {
    for empty_selected in [false, true] {
        let (mut b, d) = builder();
        let root = expanded(&mut b, &d, Namespace::svg(), None, "svg");
        let nested = expanded(&mut b, &d, Namespace::svg(), None, "g");
        let nt = expanded(&mut b, &d, Namespace::svg(), None, "title");
        let ht = html(&mut b, &d, "title");
        children(&mut b, &nested, &[nt]);
        let mut list = vec![nested, ht];
        let selected = expanded(&mut b, &d, Namespace::svg(), None, "title");
        if empty_selected {
            children(&mut b, &selected, &[]);
            list.push(selected.clone());
        }
        children(&mut b, &root, &list);
        children(&mut b, &d, &[root]);
        let c = b.finish().unwrap();
        let r = report(
            &c,
            &c.subject_id(&d).unwrap(),
            HtmlSemanticQuery::DocumentTitle,
        );
        assert!(doc_known(&r).value().is_empty());
        assert_eq!(
            doc_known(&r).title_element(),
            empty_selected
                .then(|| c.subject_id(&selected).unwrap())
                .as_ref()
        );
    }
}

#[test]
fn unknown_svg_root_children_are_missing_not_known_absence() {
    let (mut b, d) = builder();
    let root = expanded(&mut b, &d, Namespace::svg(), None, "svg");
    children(&mut b, &d, std::slice::from_ref(&root));
    let c = b.finish().unwrap();
    let r = report(
        &c,
        &c.subject_id(&d).unwrap(),
        HtmlSemanticQuery::DocumentTitle,
    );
    assert_eq!(
        doc_value(&r),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(
            c.subject_id(&root).unwrap()
        ))
    );
}

#[test]
fn non_svg_local_root_uses_html_selection_and_case_is_exact() {
    let (mut b, d) = builder();
    let root = expanded(&mut b, &d, Namespace::svg(), None, "SVG");
    let wrong = html(&mut b, &d, "Title");
    let t = html(&mut b, &d, "title");
    children(&mut b, &wrong, &[]);
    children(&mut b, &t, &[]);
    children(&mut b, &root, &[wrong, t.clone()]);
    children(&mut b, &d, &[root]);
    let c = b.finish().unwrap();
    let r = report(
        &c,
        &c.subject_id(&d).unwrap(),
        HtmlSemanticQuery::DocumentTitle,
    );
    assert_eq!(
        doc_known(&r).title_element(),
        Some(&c.subject_id(&t).unwrap())
    );
}

#[test]
fn detached_same_document_and_template_content_titles_are_excluded() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "html");
    let detached = html(&mut b, &d, "title");
    let template = html(&mut b, &d, "template");
    let fragment = b.add_fragment(&d).unwrap();
    let ft = html(&mut b, &d, "title");
    children(&mut b, &fragment, &[ft]);
    b.establish_template_contents(&template, &fragment).unwrap();
    children(&mut b, &template, &[]);
    children(&mut b, &detached, &[]);
    children(&mut b, &root, &[template]);
    children(&mut b, &d, &[root]);
    let c = b.finish().unwrap();
    let r = report(
        &c,
        &c.subject_id(&d).unwrap(),
        HtmlSemanticQuery::DocumentTitle,
    );
    assert!(doc_known(&r).value().is_empty());
    assert_eq!(doc_known(&r).title_element(), None);
}

#[test]
fn title_text_and_document_title_have_distinct_subject_domains() {
    let (mut b, d) = builder();
    let wrong = html(&mut b, &d, "Title");
    let svg = expanded(&mut b, &d, Namespace::svg(), None, "title");
    let f = b.add_fragment(&d).unwrap();
    let leaf = text(&mut b, &d, "x");
    let c = b.finish().unwrap();
    for subject in [&d, &wrong, &f, &leaf] {
        let r = report(
            &c,
            &c.subject_id(subject).unwrap(),
            HtmlSemanticQuery::TitleText,
        );
        assert_eq!(title_value(&r), &HtmlSemanticValue::Inapplicable);
    }
    assert_eq!(
        title_value(&report(
            &c,
            &c.subject_id(&svg).unwrap(),
            HtmlSemanticQuery::TitleText
        )),
        &HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
    );
    for subject in [&wrong, &svg, &f, &leaf] {
        assert_eq!(
            doc_value(&report(
                &c,
                &c.subject_id(subject).unwrap(),
                HtmlSemanticQuery::DocumentTitle
            )),
            &HtmlSemanticValue::Inapplicable
        );
    }
}

fn restricted(resource: HtmlSemanticResource, cap: usize) -> HtmlSemanticLimits {
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
fn literal_child_text_and_two_stage_normalization_work_have_exact_one_short_limits() {
    let (c, d, t) = single(" A ".into());
    let cx = context(&c);
    // Title: result reserve + request node + HTML namespace(1+28) + local(1+5)
    // + child text(4+2*1+3*3) + result slot = 53.
    // Document: reserve/request(2), unique root(5), SVG length mismatch(1),
    // ordinary descent(4), title node(1), HTML namespace/local(35), child(15),
    // normalize(3+2*1+1), readback(1), result(1) = 71.
    for (id, q, credits) in [
        (&t, HtmlSemanticQuery::TitleText, 53),
        (&d, HtmlSemanticQuery::DocumentTitle, 71),
    ] {
        for cap in [credits, credits - 1] {
            let mut s = session(
                &c,
                &cx,
                restricted(HtmlSemanticResource::Work, cap),
                HtmlSemanticCancellation::new(),
            );
            let result = s.query(&[req(id, q)]);
            if cap == credits {
                assert_eq!(result.unwrap().usage().work(), credits);
            } else {
                let f = result.unwrap_err();
                assert_eq!(
                    f.kind(),
                    &HtmlSemanticFailureKind::LimitExceeded {
                        resource: HtmlSemanticResource::Work,
                        limit: cap,
                        required: credits
                    }
                );
                assert_eq!(s.query(&[]).unwrap_err(), f);
            }
            assert_eq!(s.usage().scratch_records(), 0);
        }
    }
}

#[test]
fn both_real_buffers_and_traversal_capacity_have_exact_one_short_admission() {
    let (c, d, _) = single(" A ".into());
    let cx = context(&c);
    let r = report(&c, &d, HtmlSemanticQuery::DocumentTitle);
    let u = r.usage();
    assert_eq!(u.nested_output_records(), 4); // raw three units, normalized one.
    assert_eq!(u.peak_scratch_records(), 1); // Document iterator, title matches before descent.
    assert_eq!(
        u.allocated_bytes(),
        std::mem::size_of::<HtmlSemanticResult>()
            + std::mem::size_of::<HtmlCapturedChildIds<'_>>()
            + 8
    );
    for (resource, exact) in [
        (HtmlSemanticResource::NestedOutputRecords, 4),
        (HtmlSemanticResource::ScratchRecords, 1),
        (HtmlSemanticResource::AllocatedBytes, u.allocated_bytes()),
        (HtmlSemanticResource::OutputRecords, 1),
    ] {
        for cap in [exact, exact - 1] {
            let mut s = session(
                &c,
                &cx,
                restricted(resource, cap),
                HtmlSemanticCancellation::new(),
            );
            let result = s.query(&[req(&d, HtmlSemanticQuery::DocumentTitle)]);
            if cap == exact {
                assert!(result.is_ok());
            } else {
                let f = result.unwrap_err();
                assert!(
                    matches!(f.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r==resource)
                );
                assert_eq!(s.query(&[]).unwrap_err(), f);
            }
            assert_eq!(s.usage().scratch_records(), 0);
        }
    }
}

#[test]
fn all_whitespace_still_admits_actual_raw_materialization_and_empty_source() {
    let (c, d, t) = single(" \t\r\n ".into());
    let cx = context(&c);
    let r = report(&c, &d, HtmlSemanticQuery::DocumentTitle);
    assert_eq!(r.usage().nested_output_records(), 5);
    assert!(doc_known(&r).value().is_empty());
    assert_eq!(doc_known(&r).title_element(), Some(&t));
    let mut s = session(
        &c,
        &cx,
        restricted(HtmlSemanticResource::NestedOutputRecords, 4),
        HtmlSemanticCancellation::new(),
    );
    let f = s
        .query(&[req(&d, HtmlSemanticQuery::DocumentTitle)])
        .unwrap_err();
    assert_eq!(
        f.kind(),
        &HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::NestedOutputRecords,
            limit: 4,
            required: 5
        }
    );
    assert_eq!(s.usage().scratch_records(), 0);
}

#[test]
fn late_batch_limit_returns_no_partial_report_and_releases_actual_frames() {
    let (c, d, t) = single(" A ".into());
    let cx = context(&c);
    let mut s = session(
        &c,
        &cx,
        restricted(HtmlSemanticResource::NestedOutputRecords, 3),
        HtmlSemanticCancellation::new(),
    );
    let f = s
        .query(&[
            req(&t, HtmlSemanticQuery::TitleText),
            req(&d, HtmlSemanticQuery::DocumentTitle),
        ])
        .unwrap_err();
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

#[test]
fn repeated_readback_capacity_is_cumulative_and_prior_report_survives_stop() {
    let (c, d, _) = single(" A ".into());
    let cx = context(&c);
    let mut s = session(
        &c,
        &cx,
        restricted(HtmlSemanticResource::NestedOutputRecords, 7),
        HtmlSemanticCancellation::new(),
    );
    let first = s
        .query(&[req(&d, HtmlSemanticQuery::DocumentTitle)])
        .unwrap();
    assert_eq!(first.usage().nested_output_records(), 4);
    let f = s
        .query(&[req(&d, HtmlSemanticQuery::DocumentTitle)])
        .unwrap_err();
    assert_eq!(
        f.kind(),
        &HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::NestedOutputRecords,
            limit: 7,
            required: 8
        }
    );
    assert_eq!(doc_known(&first).value().code_units(), [65]);
    assert_eq!(s.usage().scratch_records(), 0);
}

#[test]
fn cancellation_and_foreign_subjects_preserve_first_stop_and_qualification() {
    let (c, d, _) = single(" A ".into());
    let cx = context(&c);
    let cancel = HtmlSemanticCancellation::new();
    let mut s = session(&c, &cx, limits(), cancel.clone());
    let first = s
        .query(&[req(&d, HtmlSemanticQuery::DocumentTitle)])
        .unwrap();
    let before = s.usage();
    cancel.cancel();
    let f = s
        .query(&[req(&d, HtmlSemanticQuery::DocumentTitle)])
        .unwrap_err();
    assert_eq!(f.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(s.usage(), before);
    assert_eq!(s.query(&[]).unwrap_err(), f);
    assert_eq!(first.qualification().capture(), c.identity());
    assert_eq!(first.qualification().context(), cx.identity());
    assert_eq!(
        first.qualification().host_revision(),
        c.host_facts_revision()
    );
    assert_eq!(first.qualification().attribute_input_limit(), 0);
    assert_eq!(first.qualification().limits(), limits());
    assert_eq!(doc_known(&first).value().code_units(), [65]);
    let (other, foreign, _) = single("y".into());
    assert!(
        matches!(HtmlSemanticSession::new(&other,&cx,HtmlValueInterpreter::new(0),limits(),HtmlSemanticCancellation::new()),Err(e) if *e.kind()==HtmlSemanticFailureKind::ContextCaptureMismatch)
    );
    let mut s = session(&c, &cx, limits(), HtmlSemanticCancellation::new());
    let f = s
        .query(&[req(&foreign, HtmlSemanticQuery::DocumentTitle)])
        .unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Capture(_)));
    assert_eq!(s.query(&[]).unwrap_err(), f);
}

#[test]
fn source_clauses_identify_separate_getters() {
    assert!(
        HtmlSemanticQuery::TitleText
            .source_clause()
            .ends_with("#dom-title-text")
    );
    assert!(
        HtmlSemanticQuery::DocumentTitle
            .source_clause()
            .ends_with("#document.title")
    );
}

#[test]
fn shadow_root_title_never_enters_document_selection() {
    let (mut b, d) = builder();
    let host = html(&mut b, &d, "div");
    let shadow = b
        .add_shadow_root(
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
        .unwrap();
    let t = html(&mut b, &d, "title");
    let a = text(&mut b, &d, "shadow title");
    children(&mut b, &t, &[a]);
    children(&mut b, &shadow, &[t]);
    children(&mut b, &host, &[]);
    children(&mut b, &d, &[host]);
    let c = b.finish().unwrap();
    let r = report(
        &c,
        &c.subject_id(&d).unwrap(),
        HtmlSemanticQuery::DocumentTitle,
    );
    assert!(doc_known(&r).value().is_empty());
    assert_eq!(doc_known(&r).title_element(), None);
}

#[test]
fn nested_element_text_is_not_child_text_or_normalized_document_text() {
    let (mut b, d) = builder();
    let t = html(&mut b, &d, "title");
    let a = text(&mut b, &d, "a");
    let span = html(&mut b, &d, "span");
    let nested = text(&mut b, &d, " should not be read ");
    let z = text(&mut b, &d, "b");
    children(&mut b, &span, &[nested]);
    children(&mut b, &t, &[a, span, z]);
    children(&mut b, &d, std::slice::from_ref(&t));
    let c = b.finish().unwrap();
    let ti = c.subject_id(&t).unwrap();
    let di = c.subject_id(&d).unwrap();
    assert_eq!(
        title_value(&report(&c, &ti, HtmlSemanticQuery::TitleText)),
        &HtmlSemanticValue::Known("ab".into())
    );
    let r = report(&c, &di, HtmlSemanticQuery::DocumentTitle);
    assert_eq!(doc_known(&r).value().code_units(), [97, 98]);
    assert_eq!(r.usage().nested_output_records(), 4);
}

#[test]
fn exact_namespace_uri_identity_accepts_custom_storage_and_rejects_no_namespace_title() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "html");
    let wrong = expanded(&mut b, &d, Namespace::none(), None, "title");
    let exact = expanded(
        &mut b,
        &d,
        Namespace::from_uri("http://www.w3.org/1999/xhtml"),
        Some("q"),
        "title",
    );
    children(&mut b, &wrong, &[]);
    children(&mut b, &exact, &[]);
    children(&mut b, &root, &[wrong, exact.clone()]);
    children(&mut b, &d, &[root]);
    let c = b.finish().unwrap();
    let r = report(
        &c,
        &c.subject_id(&d).unwrap(),
        HtmlSemanticQuery::DocumentTitle,
    );
    assert_eq!(
        doc_known(&r).title_element(),
        Some(&c.subject_id(&exact).unwrap())
    );
    assert_eq!(
        title_value(&report(
            &c,
            &c.subject_id(&exact).unwrap(),
            HtmlSemanticQuery::TitleText
        )),
        &HtmlSemanticValue::Known(DocumentString::default())
    );
}
