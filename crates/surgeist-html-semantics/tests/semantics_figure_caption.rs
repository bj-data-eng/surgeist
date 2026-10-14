use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlCapturedShadowProperties,
    HtmlSemanticCancellation, HtmlSemanticDependencies, HtmlSemanticFact, HtmlSemanticFailureKind,
    HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport,
    HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticResult, HtmlSemanticSession,
    HtmlSemanticUnsupported, HtmlSemanticValue,
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
#[path = "common/scalar_limits.rs"]
mod shared_scalar_limits;
use shared_scalar_limits::scalar_limits as resource_limits;
use surgeist_html::*;

fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    ns: Namespace,
    local: &str,
    attrs: OrderedAttributes,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(ns, None, local),
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
        Namespace::html(),
        local,
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
    HtmlSemanticLimits::new(20000000, 100, 1000, 1000000, 1000000, 100000)
}

fn request(id: &HtmlCaptureSubjectId, q: HtmlSemanticQuery) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), q)
}
fn query(
    c: &HtmlCapture,
    id: &HtmlCaptureSubjectId,
    q: HtmlSemanticQuery,
    raw: usize,
) -> HtmlSemanticReport {
    let cx = context(c);
    session(c, &cx, limits(), raw, HtmlSemanticCancellation::new())
        .query(&[request(id, q)])
        .unwrap()
}

fn value(r: &HtmlSemanticReport) -> &HtmlSemanticValue<Option<HtmlCaptureSubjectId>> {
    match r.results()[0].fact() {
        HtmlSemanticFact::FigureCaption(value) => value,
        other => panic!("expected figure caption, got {other:?}"),
    }
}
fn report(c: &HtmlCapture, id: &HtmlCaptureSubjectId) -> HtmlSemanticReport {
    query(c, id, HtmlSemanticQuery::FigureCaption, 0)
}
fn simple() -> (HtmlCapture, HtmlCaptureSubjectId, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let figure = html(&mut b, &d, "figure", &[]);
    let caption = html(&mut b, &d, "figcaption", &[]);
    children(&mut b, &figure, std::slice::from_ref(&caption));
    let c = b.finish().unwrap();
    let id = c.subject_id(&figure).unwrap();
    let expected = c.subject_id(&caption).unwrap();
    (c, id, expected)
}

#[test]
fn first_caption_in_the_middle_wins_even_when_author_placement_is_invalid() {
    let (mut b, d) = builder();
    let figure = html(&mut b, &d, "figure", &[]);
    let before = html(&mut b, &d, "div", &[]);
    let first = html(&mut b, &d, "figcaption", &[]);
    let after = html(&mut b, &d, "div", &[]);
    let second = html(&mut b, &d, "figcaption", &[]);
    children(&mut b, &figure, &[before, first.clone(), after, second]);
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&figure).unwrap())),
        &HtmlSemanticValue::Known(Some(c.subject_id(&first).unwrap()))
    );
}

#[test]
fn caption_first_does_not_require_its_children_or_later_branches() {
    let (c, id, expected) = simple();
    let r = report(&c, &id);
    assert_eq!(value(&r), &HtmlSemanticValue::Known(Some(expected)));
    assert_eq!(r.usage().scratch_records(), 0);
    assert_eq!(r.usage().nested_output_records(), 0);
}

#[test]
fn caption_last_is_selected_after_all_noncaption_direct_children() {
    let (mut b, d) = builder();
    let figure = html(&mut b, &d, "figure", &[]);
    let before = html(&mut b, &d, "div", &[]);
    let caption = html(&mut b, &d, "figcaption", &[]);
    children(&mut b, &figure, &[before, caption.clone()]);
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&figure).unwrap())),
        &HtmlSemanticValue::Known(Some(c.subject_id(&caption).unwrap()))
    );
}

#[test]
fn no_namespace_and_utf16_lookalikes_do_not_supply_a_caption() {
    let (mut b, d) = builder();
    let figure = html(&mut b, &d, "figure", &[]);
    let none = element(
        &mut b,
        &d,
        Namespace::none(),
        "figcaption",
        OrderedAttributes::new([]).unwrap(),
    );
    let lookalike = html(&mut b, &d, "figcaptıon", &[]);
    let malformed = b
        .add_element(
            &d,
            QualifiedName::new(
                Namespace::html(),
                None,
                DocumentString::from_utf16(vec![0x66, 0x69, 0x67, 0xd800]),
            ),
            OrderedAttributes::new([]).unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    children(&mut b, &figure, &[none, lookalike, malformed]);
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&figure).unwrap())),
        &HtmlSemanticValue::Known(None)
    );
}

#[test]
fn a_first_caption_does_not_consult_a_later_long_element_name() {
    let (mut b, d) = builder();
    let figure = html(&mut b, &d, "figure", &[]);
    let caption = html(&mut b, &d, "figcaption", &[]);
    let later = html(&mut b, &d, &"x".repeat(10000), &[]);
    children(&mut b, &figure, &[caption.clone(), later]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&figure).unwrap();
    let (baseline, bid, _) = simple();
    let cx = context(&c);
    let r = session(
        &c,
        &cx,
        resource_limits(
            HtmlSemanticResource::Work,
            report(&baseline, &bid).usage().work(),
        ),
        0,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(&id, HtmlSemanticQuery::FigureCaption)])
    .unwrap();
    assert_eq!(
        value(&r),
        &HtmlSemanticValue::Known(Some(c.subject_id(&caption).unwrap()))
    );
}

#[test]
fn changed_child_order_recomputes_under_distinct_qualifications_and_preserves_old_reports() {
    let mut reports = Vec::new();
    for reverse in [false, true] {
        let (mut b, d) = builder();
        let figure = html(&mut b, &d, "figure", &[]);
        let a = html(&mut b, &d, "figcaption", &[]);
        let z = html(&mut b, &d, "figcaption", &[]);
        let expected = if reverse { &z } else { &a };
        let ordered = if reverse {
            [z.clone(), a.clone()]
        } else {
            [a.clone(), z.clone()]
        };
        children(&mut b, &figure, &ordered);
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&figure).unwrap());
        let expected_id = c.subject_id(expected).unwrap();
        assert_eq!(
            value(&r),
            &HtmlSemanticValue::Known(Some(expected_id.clone()))
        );
        reports.push((r, expected_id));
    }
    assert_ne!(reports[0].0.qualification(), reports[1].0.qualification());
    for (r, id) in reports {
        assert_eq!(value(&r), &HtmlSemanticValue::Known(Some(id)));
    }
}

#[test]
fn text_comment_and_processing_instruction_are_not_captions() {
    let (mut b, d) = builder();
    let figure = html(&mut b, &d, "figure", &[]);
    let t = b
        .add_text(&d, DocumentString::from_utf16(vec![0xd800, 10]))
        .unwrap();
    let comment = b.add_comment(&d, "figcaption".into()).unwrap();
    let pi = b
        .add_processing_instruction(&d, "figcaption".into(), "huge unused".into())
        .unwrap();
    let caption = html(&mut b, &d, "figcaption", &[]);
    children(&mut b, &figure, &[t, comment, pi, caption.clone()]);
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&figure).unwrap())),
        &HtmlSemanticValue::Known(Some(c.subject_id(&caption).unwrap()))
    );
}

#[test]
fn only_nested_caption_is_not_a_figure_caption() {
    let (mut b, d) = builder();
    let figure = html(&mut b, &d, "figure", &[]);
    let wrapper = html(&mut b, &d, "div", &[]);
    let nested = html(&mut b, &d, "figcaption", &[]);
    children(&mut b, &wrapper, &[nested]);
    children(&mut b, &figure, &[wrapper]);
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&figure).unwrap())),
        &HtmlSemanticValue::Known(None)
    );
}

#[test]
fn complete_empty_detached_figure_is_known_without_a_caption() {
    let (mut b, d) = builder();
    let figure = html(&mut b, &d, "figure", &[]);
    children(&mut b, &figure, &[]);
    b.establish_root(&figure).unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&figure).unwrap())),
        &HtmlSemanticValue::Known(None)
    );
}

#[test]
fn unknown_order_is_missing_but_unknown_descendants_are_irrelevant() {
    let (mut b, d) = builder();
    let missing = html(&mut b, &d, "figure", &[]);
    let complete = html(&mut b, &d, "figure", &[]);
    let div = html(&mut b, &d, "div", &[]);
    children(&mut b, &complete, &[div]);
    let c = b.finish().unwrap();
    let mid = c.subject_id(&missing).unwrap();
    assert_eq!(
        value(&report(&c, &mid)),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(mid))
    );
    assert_eq!(
        value(&report(&c, &c.subject_id(&complete).unwrap())),
        &HtmlSemanticValue::Known(None)
    );
}

#[test]
fn expanded_name_matches_ignore_prefix_and_skip_foreign_and_uppercase_children() {
    let (mut b, d) = builder();
    let figure = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), Some("x".into()), "figure"),
            OrderedAttributes::new([]).unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let foreign = element(
        &mut b,
        &d,
        Namespace::svg(),
        "figcaption",
        OrderedAttributes::new([]).unwrap(),
    );
    let upper = html(&mut b, &d, "FIGCAPTION", &[]);
    let exact = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), Some("y".into()), "figcaption"),
            OrderedAttributes::new([]).unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    children(&mut b, &figure, &[foreign, upper, exact.clone()]);
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&figure).unwrap())),
        &HtmlSemanticValue::Known(Some(c.subject_id(&exact).unwrap()))
    );
}

#[test]
fn wrong_role_non_element_and_foreign_namespace_preserve_applicability() {
    let (mut b, d) = builder();
    let div = html(&mut b, &d, "div", &[]);
    let upper = html(&mut b, &d, "FIGURE", &[]);
    let foreign = element(
        &mut b,
        &d,
        Namespace::mathml(),
        "figure",
        OrderedAttributes::new([]).unwrap(),
    );
    let c = b.finish().unwrap();
    for id in [&div, &upper, &d] {
        assert_eq!(
            value(&report(&c, &c.subject_id(id).unwrap())),
            &HtmlSemanticValue::Inapplicable
        );
    }
    assert_eq!(
        value(&report(&c, &c.subject_id(&foreign).unwrap())),
        &HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
    );
}

#[test]
fn hosted_template_and_shadow_contents_never_supply_direct_captions() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let figure = html(&mut b, &d, "figure", &[]);
        let host = html(&mut b, &d, if shadow { "div" } else { "template" }, &[]);
        let f = if shadow {
            b.add_shadow_root(
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
        let hidden = html(&mut b, &d, "figcaption", &[]);
        children(&mut b, &f, &[hidden]);
        children(&mut b, &figure, &[host]);
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&figure).unwrap())),
            &HtmlSemanticValue::Known(None)
        );
    }
}

#[test]
fn huge_title_and_text_are_unconsulted_with_zero_raw_scratch_and_nested_caps() {
    let (mut b, d) = builder();
    let huge = "x".repeat(10000);
    let figure = html(&mut b, &d, "figure", &[("title", &huge)]);
    let t = b.add_text(&d, huge.into()).unwrap();
    let caption = html(&mut b, &d, "figcaption", &[("title", "unread")]);
    children(&mut b, &figure, &[t, caption.clone()]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let r = session(
        &c,
        &cx,
        HtmlSemanticLimits::new(20000000, 1, 0, 0, 1000000, 0),
        0,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(
        &c.subject_id(&figure).unwrap(),
        HtmlSemanticQuery::FigureCaption,
    )])
    .unwrap();
    assert_eq!(
        value(&r),
        &HtmlSemanticValue::Known(Some(c.subject_id(&caption).unwrap()))
    );
    assert_eq!(r.usage().peak_scratch_records(), 0);
    assert_eq!(r.usage().nested_output_records(), 0);
}

#[test]
fn exact_and_one_short_work_output_and_actual_bytes_retain_the_first_stop() {
    let (c, id, expected) = simple();
    let cx = context(&c);
    let u = report(&c, &id).usage();
    assert_eq!(u.output_records(), 1);
    assert_eq!(u.nested_output_records(), 0);
    assert_eq!(u.peak_scratch_records(), 0);
    assert!(u.allocated_bytes() >= std::mem::size_of::<HtmlSemanticResult>());
    for (resource, exact) in [
        (HtmlSemanticResource::Work, u.work()),
        (HtmlSemanticResource::OutputRecords, u.output_records()),
        (HtmlSemanticResource::AllocatedBytes, u.allocated_bytes()),
    ] {
        let req = request(&id, HtmlSemanticQuery::FigureCaption);
        let r = session(
            &c,
            &cx,
            resource_limits(resource, exact),
            0,
            HtmlSemanticCancellation::new(),
        )
        .query(std::slice::from_ref(&req))
        .unwrap();
        assert_eq!(value(&r), &HtmlSemanticValue::Known(Some(expected.clone())));
        let mut s = session(
            &c,
            &cx,
            resource_limits(resource, exact - 1),
            0,
            HtmlSemanticCancellation::new(),
        );
        let failure = s.query(std::slice::from_ref(&req)).unwrap_err();
        assert!(
            matches!(failure.kind(), HtmlSemanticFailureKind::LimitExceeded { resource: actual, .. } if *actual == resource)
        );
        assert_eq!(s.query(&[]).unwrap_err(), failure);
        assert_eq!(s.usage().scratch_records(), 0);
        assert_eq!(failure.qualification().unwrap().capture(), c.identity());
    }
}

#[test]
fn repeated_queries_charge_again_and_late_work_failure_returns_no_batch_prefix() {
    let (c, id, expected) = simple();
    let cx = context(&c);
    let req = request(&id, HtmlSemanticQuery::FigureCaption);
    let mut s = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let old = s.query(std::slice::from_ref(&req)).unwrap();
    let again = s.query(std::slice::from_ref(&req)).unwrap();
    assert_eq!(again.usage().work(), 2 * old.usage().work());
    assert_eq!(
        again.usage().allocated_bytes(),
        2 * old.usage().allocated_bytes()
    );
    assert_eq!(value(&old), &HtmlSemanticValue::Known(Some(expected)));
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
    assert_eq!(value(&old), value(&again));
}

#[test]
fn cancellation_is_sticky_and_does_not_invalidate_an_earlier_report() {
    let (c, id, expected) = simple();
    let cx = context(&c);
    let cancel = HtmlSemanticCancellation::new();
    let mut s = session(&c, &cx, limits(), 0, cancel.clone());
    let old = s
        .query(&[request(&id, HtmlSemanticQuery::FigureCaption)])
        .unwrap();
    cancel.cancel();
    let f = s
        .query(&[request(&id, HtmlSemanticQuery::FigureCaption)])
        .unwrap_err();
    assert_eq!(f.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(f.qualification().unwrap(), old.qualification());
    assert_eq!(s.query(&[]).unwrap_err(), f);
    assert_eq!(value(&old), &HtmlSemanticValue::Known(Some(expected)));
    assert_eq!(s.usage().scratch_records(), 0);
}

#[test]
fn capture_context_host_and_config_are_exact_and_foreign_subjects_fail_atomically() {
    let (c, id, _) = simple();
    let (other, oid, _) = simple();
    let cx = context(&c);
    let other_cx = context(&other);
    let r = report(&c, &id);
    let or = report(&other, &oid);
    assert_ne!(r.capture(), or.capture());
    assert_ne!(r.host_revision(), or.host_revision());
    assert_eq!(
        r.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
    let extra = context(&c);
    let rr = session(&c, &extra, limits(), 1, HtmlSemanticCancellation::new())
        .query(&[request(&id, HtmlSemanticQuery::FigureCaption)])
        .unwrap();
    assert_ne!(r.context(), rr.context());
    assert_ne!(r.qualification(), rr.qualification());
    assert_eq!(rr.qualification().attribute_input_limit(), 1);
    assert!(
        HtmlSemanticSession::new(
            &c,
            &other_cx,
            HtmlValueInterpreter::new(0),
            limits(),
            HtmlSemanticCancellation::new()
        )
        .is_err()
    );
    let mut s = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let f = s
        .query(&[
            request(&id, HtmlSemanticQuery::FigureCaption),
            request(&oid, HtmlSemanticQuery::FigureCaption),
        ])
        .unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Capture(_)));
    assert_eq!(s.query(&[]).unwrap_err(), f);
}
