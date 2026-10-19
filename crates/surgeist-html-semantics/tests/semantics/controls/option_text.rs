use crate::semantics::support::{builder, semantic_limits as limits, session};
use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlCapturedShadowProperties, HtmlControlFacts,
    HtmlControlFactsLimits, HtmlControlFlag, HtmlOptionControlState, HtmlOptionReadback,
    HtmlOptionReadbackOrigin, HtmlSelectValue, HtmlSemanticCancellation, HtmlSemanticContext,
    HtmlSemanticContextInput, HtmlSemanticFact, HtmlSemanticFailureKind, HtmlSemanticLimits,
    HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticRequest, HtmlSemanticResource,
    HtmlSemanticUnsupported, HtmlSemanticValue,
};

fn attrs(values: &[(&str, &str)]) -> OrderedAttributes {
    OrderedAttributes::new(
        values
            .iter()
            .map(|(n, v)| Attribute::new(QualifiedName::new(Namespace::none(), None, *n), *v)),
    )
    .unwrap()
}
fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    ns: Namespace,
    local: &str,
    attributes: OrderedAttributes,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(ns, None, local),
        attributes,
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn html(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    local: &str,
    values: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    element(b, d, Namespace::html(), local, attrs(values))
}
fn children(
    b: &mut HtmlCaptureBuilder,
    parent: &HtmlCaptureBuilderSubject,
    items: &[HtmlCaptureBuilderSubject],
) {
    b.establish_children(parent, items).unwrap();
}
fn text(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    value: impl Into<DocumentString>,
) -> HtmlCaptureBuilderSubject {
    b.add_text(d, value.into()).unwrap()
}
fn context(c: &HtmlCapture, flags: &[(HtmlCaptureSubjectId, Option<bool>)]) -> HtmlSemanticContext {
    let controls = HtmlControlFacts::new(
        c,
        c.host_facts_revision().clone(),
        vec![],
        vec![],
        flags
            .iter()
            .map(|(id, v)| HtmlOptionControlState::new(id.clone(), *v, None))
            .collect(),
        vec![],
        HtmlControlFactsLimits::new(1000, 1000000),
    )
    .unwrap();
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_controls(controls),
        0,
    )
    .unwrap()
}
fn request(id: &HtmlCaptureSubjectId, q: HtmlSemanticQuery) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), q)
}
fn query(
    c: &HtmlCapture,
    cx: &HtmlSemanticContext,
    id: &HtmlCaptureSubjectId,
    q: HtmlSemanticQuery,
) -> HtmlSemanticFact {
    session(c, cx, limits(), 100000, HtmlSemanticCancellation::new())
        .query(&[request(id, q)])
        .unwrap()
        .results()[0]
        .fact()
        .clone()
}
fn option(f: &HtmlSemanticFact) -> &HtmlOptionReadback {
    match f {
        HtmlSemanticFact::OptionText(HtmlSemanticValue::Known(v))
        | HtmlSemanticFact::OptionLabel(HtmlSemanticValue::Known(v))
        | HtmlSemanticFact::OptionValue(HtmlSemanticValue::Known(v)) => v,
        other => panic!("expected option readback, got {other:?}"),
    }
}
fn select(f: &HtmlSemanticFact) -> &HtmlSelectValue {
    match f {
        HtmlSemanticFact::SelectValue(HtmlSemanticValue::Known(v)) => v,
        other => panic!("expected select value, got {other:?}"),
    }
}
fn leaf_option(
    values: &[(&str, &str)],
    texts: &[DocumentString],
) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let o = html(&mut b, &d, "option", values);
    let ts: Vec<_> = texts.iter().map(|v| text(&mut b, &d, v.clone())).collect();
    children(&mut b, &o, &ts);
    b.establish_root(&o).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&o).unwrap();
    (c, id)
}

#[test]
fn collapse_is_across_text_boundaries_and_only_five_ascii_whitespace_units() {
    let (c, o) = leaf_option(&[], &[" \tA".into(), "\n\r".into(), "B\u{c} ".into()]);
    let cx = context(&c, &[]);
    for q in [
        HtmlSemanticQuery::OptionText,
        HtmlSemanticQuery::OptionLabel,
        HtmlSemanticQuery::OptionValue,
    ] {
        let fact = query(&c, &cx, &o, q);
        assert_eq!(option(&fact).value(), &DocumentString::from("A B"));
        assert_eq!(
            option(&fact).origin(),
            HtmlOptionReadbackOrigin::DescendantText
        );
    }
    let raw = DocumentString::from_utf16(vec![
        0x20, 0xa0, 0x0b, 0x2003, 0, 0xd800, 0xd83d, 0xde00, 0x20,
    ]);
    let (c, o) = leaf_option(&[], &[raw]);
    assert_eq!(
        option(&query(
            &c,
            &context(&c, &[]),
            &o,
            HtmlSemanticQuery::OptionText
        ))
        .value()
        .code_units(),
        [0xa0, 0x0b, 0x2003, 0, 0xd800, 0xd83d, 0xde00]
    );
}

#[test]
fn nested_text_concatenates_without_br_or_element_separators() {
    let (mut b, d) = builder();
    let o = html(&mut b, &d, "option", &[]);
    let a = text(&mut b, &d, "A");
    let wrap = html(&mut b, &d, "div", &[]);
    let br = html(&mut b, &d, "br", &[]);
    let nested = html(&mut b, &d, "option", &[]);
    let bt = text(&mut b, &d, "B");
    let ct = text(&mut b, &d, "C");
    children(&mut b, &wrap, &[bt]);
    children(&mut b, &br, &[]);
    children(&mut b, &nested, &[ct]);
    children(&mut b, &o, &[a, wrap, br, nested]);
    let c = b.finish().unwrap();
    assert_eq!(
        option(&query(
            &c,
            &context(&c, &[]),
            &c.subject_id(&o).unwrap(),
            HtmlSemanticQuery::OptionText
        ))
        .value(),
        &DocumentString::from("ABC")
    );
}

#[test]
fn script_and_image_subtrees_are_pruned_by_exact_expanded_names() {
    let (mut b, d) = builder();
    let o = html(&mut b, &d, "option", &[]);
    let a = text(&mut b, &d, "A");
    // Deliberately unavailable children and huge ignored alt values.
    let script = html(&mut b, &d, "script", &[]);
    let svgscript = element(&mut b, &d, Namespace::svg(), "script", attrs(&[]));
    let img = html(&mut b, &d, "img", &[("alt", "ignored alt")]);
    let mut included = Vec::new();
    for (ns, name, value) in [
        (Namespace::svg(), "img", "B"),
        (Namespace::html(), "SCRIPT", "C"),
        (Namespace::mathml(), "script", "D"),
        (
            Namespace::from_uri("http://www.w3.org/2000/SVG"),
            "script",
            "E",
        ),
    ] {
        let e = element(&mut b, &d, ns, name, attrs(&[]));
        let t = text(&mut b, &d, value);
        children(&mut b, &e, &[t]);
        included.push(e);
    }
    let comment = b.add_comment(&d, "ignored comment".into()).unwrap();
    let mut nodes = vec![a, script, svgscript, img, comment];
    nodes.extend(included);
    children(&mut b, &o, &nodes);
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    let r = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
        .query(&[request(
            &c.subject_id(&o).unwrap(),
            HtmlSemanticQuery::OptionText,
        )])
        .unwrap();
    assert_eq!(
        option(r.results()[0].fact()).value(),
        &DocumentString::from("ABCDE")
    );
}

#[test]
fn idl_label_and_value_present_empty_do_not_fall_back_but_text_always_does() {
    for values in [
        vec![],
        vec![("label", ""), ("value", "")],
        vec![("label", " \tRaw Label "), ("value", " \nRaw Value ")],
    ] {
        let (c, o) = leaf_option(&values, &[" collapsed  text ".into()]);
        let cx = context(&c, &[]);
        let t = query(&c, &cx, &o, HtmlSemanticQuery::OptionText);
        assert_eq!(option(&t).value(), &DocumentString::from("collapsed text"));
        assert_eq!(
            option(&t).origin(),
            HtmlOptionReadbackOrigin::DescendantText
        );
        for (q, name, origin) in [
            (
                HtmlSemanticQuery::OptionLabel,
                "label",
                HtmlOptionReadbackOrigin::LabelAttribute,
            ),
            (
                HtmlSemanticQuery::OptionValue,
                "value",
                HtmlOptionReadbackOrigin::ValueAttribute,
            ),
        ] {
            let fact = query(&c, &cx, &o, q);
            match values.iter().find(|(n, _)| *n == name) {
                Some((_, v)) => {
                    assert_eq!(option(&fact).value(), &DocumentString::from(*v));
                    assert_eq!(option(&fact).origin(), origin);
                }
                None => {
                    assert_eq!(
                        option(&fact).value(),
                        &DocumentString::from("collapsed text")
                    );
                    assert_eq!(
                        option(&fact).origin(),
                        HtmlOptionReadbackOrigin::DescendantText
                    );
                }
            }
        }
    }
}

#[test]
fn exact_raw_attribute_units_are_retained_without_children_and_admitted_only_when_consulted() {
    let (mut b, d) = builder();
    let raw = DocumentString::from_utf16(vec![0x20, 0xd800, 0, 0x41, 0x20]);
    let o = element(
        &mut b,
        &d,
        Namespace::html(),
        "option",
        OrderedAttributes::new([
            Attribute::new(
                QualifiedName::new(Namespace::none(), Some("p".into()), "label"),
                raw.clone(),
            ),
            Attribute::new(
                QualifiedName::new(Namespace::none(), None, "value"),
                "enormous unused value",
            ),
        ])
        .unwrap(),
    );
    let c = b.finish().unwrap();
    let id = c.subject_id(&o).unwrap();
    let cx = context(&c, &[]);
    let req = request(&id, HtmlSemanticQuery::OptionLabel);
    let r = session(&c, &cx, limits(), 5, HtmlSemanticCancellation::new())
        .query(std::slice::from_ref(&req))
        .unwrap();
    assert_eq!(option(r.results()[0].fact()).value(), &raw);
    assert_eq!(r.usage().nested_output_records(), 0);
    assert_eq!(r.usage().peak_scratch_records(), 0);
    let failure = session(&c, &cx, limits(), 4, HtmlSemanticCancellation::new())
        .query(&[req])
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    assert_eq!(
        query(&c, &cx, &id, HtmlSemanticQuery::OptionText),
        HtmlSemanticFact::OptionText(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::OrderedChildren(id)
        ))
    );
}

#[test]
fn namespaced_and_uppercase_attribute_lookalikes_are_ignored() {
    let (mut b, d) = builder();
    let o = element(
        &mut b,
        &d,
        Namespace::html(),
        "option",
        OrderedAttributes::new([
            Attribute::new(
                QualifiedName::new(Namespace::from_uri("urn:x"), None, "label"),
                "wrong",
            ),
            Attribute::new(
                QualifiedName::new(Namespace::none(), None, "VALUE"),
                "wrong",
            ),
        ])
        .unwrap(),
    );
    let t = text(&mut b, &d, "text");
    children(&mut b, &o, &[t]);
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    for q in [
        HtmlSemanticQuery::OptionLabel,
        HtmlSemanticQuery::OptionValue,
    ] {
        assert_eq!(
            option(&query(&c, &cx, &c.subject_id(&o).unwrap(), q)).value(),
            &DocumentString::from("text")
        );
    }
}

#[test]
fn complete_empty_and_all_whitespace_text_need_no_nested_output() {
    for texts in [vec![], vec![DocumentString::from(" \t\n\r\u{c}")]] {
        let (c, o) = leaf_option(&[], &texts);
        let cx = context(&c, &[]);
        let l = HtmlSemanticLimits::new(limits().work(), 1, 100, 0, 100000, 0);
        let r = session(&c, &cx, l, 0, HtmlSemanticCancellation::new())
            .query(&[request(&o, HtmlSemanticQuery::OptionText)])
            .unwrap();
        assert!(option(r.results()[0].fact()).value().is_empty());
        assert_eq!(r.usage().nested_output_records(), 0);
    }
}

fn simple_select() -> (HtmlCapture, HtmlCaptureSubjectId, [HtmlCaptureSubjectId; 3]) {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[]);
    let a = html(
        &mut b,
        &d,
        "option",
        &[("value", "first"), ("selected", "")],
    );
    let ob = html(&mut b, &d, "option", &[("value", ""), ("disabled", "")]);
    let oc = html(&mut b, &d, "option", &[("value", "third")]);
    children(&mut b, &s, &[a.clone(), ob.clone(), oc.clone()]);
    let c = b.finish().unwrap();
    let sid = c.subject_id(&s).unwrap();
    let ids = [a, ob, oc].map(|o| c.subject_id(&o).unwrap());
    (c, sid, ids)
}

#[test]
fn select_value_uses_current_first_selected_and_distinguishes_selected_empty_from_none() {
    let (c, s, os) = simple_select();
    let cx = context(
        &c,
        &[
            (os[0].clone(), Some(false)),
            (os[1].clone(), Some(true)),
            (os[2].clone(), None),
        ],
    );
    let f = query(&c, &cx, &s, HtmlSemanticQuery::SelectValue);
    assert!(select(&f).value().is_empty());
    assert_eq!(select(&f).selected_option(), Some(&os[1]));
    let cx = context(
        &c,
        &os.iter()
            .map(|o| (o.clone(), Some(false)))
            .collect::<Vec<_>>(),
    );
    let f = query(&c, &cx, &s, HtmlSemanticQuery::SelectValue);
    assert!(select(&f).value().is_empty());
    assert_eq!(select(&f).selected_option(), None);
    let cx = context(&c, &[(os[0].clone(), None), (os[1].clone(), Some(true))]);
    assert_eq!(
        query(&c, &cx, &s, HtmlSemanticQuery::SelectValue),
        HtmlSemanticFact::SelectValue(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::ControlState {
                subject: os[0].clone(),
                flag: HtmlControlFlag::Selectedness
            }
        ))
    );
}

#[test]
fn first_selection_skips_later_topology_state_and_raw_but_requires_selected_option_value() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[]);
    let a = html(&mut b, &d, "option", &[("value", "ok")]);
    let later = html(&mut b, &d, "div", &[("value", "unused enormous raw")]);
    children(&mut b, &s, &[a.clone(), later]);
    let c = b.finish().unwrap();
    let sid = c.subject_id(&s).unwrap();
    let aid = c.subject_id(&a).unwrap();
    let cx = context(&c, &[(aid.clone(), Some(true))]);
    let r = session(&c, &cx, limits(), 2, HtmlSemanticCancellation::new())
        .query(&[request(&sid, HtmlSemanticQuery::SelectValue)])
        .unwrap();
    assert_eq!(
        select(r.results()[0].fact()).value(),
        &DocumentString::from("ok")
    );
    assert_eq!(select(r.results()[0].fact()).selected_option(), Some(&aid));
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[]);
    let a = html(&mut b, &d, "option", &[]);
    children(&mut b, &s, std::slice::from_ref(&a));
    let c = b.finish().unwrap();
    let aid = c.subject_id(&a).unwrap();
    let cx = context(&c, &[(aid.clone(), Some(true))]);
    assert_eq!(
        query(
            &c,
            &cx,
            &c.subject_id(&s).unwrap(),
            HtmlSemanticQuery::SelectValue
        ),
        HtmlSemanticFact::SelectValue(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::OrderedChildren(aid)
        ))
    );
}

#[test]
fn selected_value_without_attribute_uses_shared_text_normalization() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[]);
    let o = html(&mut b, &d, "option", &[]);
    let t = text(&mut b, &d, "  selected\n text ");
    children(&mut b, &o, &[t]);
    children(&mut b, &s, std::slice::from_ref(&o));
    let c = b.finish().unwrap();
    let oid = c.subject_id(&o).unwrap();
    let cx = context(&c, &[(oid.clone(), Some(true))]);
    let f = query(
        &c,
        &cx,
        &c.subject_id(&s).unwrap(),
        HtmlSemanticQuery::SelectValue,
    );
    assert_eq!(select(&f).value(), &DocumentString::from("selected text"));
    assert_eq!(select(&f).selected_option(), Some(&oid));
}

#[test]
fn missing_after_a_text_prefix_is_not_a_partial_string_and_scratch_is_released() {
    let (mut b, d) = builder();
    let o = html(&mut b, &d, "option", &[]);
    let t = text(&mut b, &d, "prefix");
    let unknown = html(&mut b, &d, "div", &[]);
    children(&mut b, &o, &[t, unknown.clone()]);
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    let r = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
        .query(&[request(
            &c.subject_id(&o).unwrap(),
            HtmlSemanticQuery::OptionText,
        )])
        .unwrap();
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::OptionText(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::OrderedChildren(c.subject_id(&unknown).unwrap())
        ))
    );
    assert_eq!(r.usage().scratch_records(), 0);
    assert!(r.usage().nested_output_records() >= 6);
}

#[test]
fn work_output_scratch_utf16_capacity_and_bytes_have_exact_one_short_limits() {
    let (c, o) = leaf_option(&[], &[" A  B C ".into()]);
    let cx = context(&c, &[]);
    let req = request(&o, HtmlSemanticQuery::OptionText);
    let baseline = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
        .query(std::slice::from_ref(&req))
        .unwrap()
        .usage();
    assert!(baseline.nested_output_records() >= 5);
    for (resource, exact) in [
        (HtmlSemanticResource::Work, baseline.work()),
        (
            HtmlSemanticResource::OutputRecords,
            baseline.output_records(),
        ),
        (
            HtmlSemanticResource::ScratchRecords,
            baseline.peak_scratch_records(),
        ),
        (
            HtmlSemanticResource::NestedOutputRecords,
            baseline.nested_output_records(),
        ),
        (
            HtmlSemanticResource::AllocatedBytes,
            baseline.allocated_bytes(),
        ),
    ] {
        assert!(exact > 0);
        for cap in [exact, exact - 1] {
            let l = HtmlSemanticLimits::new(
                if resource == HtmlSemanticResource::Work {
                    cap
                } else {
                    limits().work()
                },
                if resource == HtmlSemanticResource::OutputRecords {
                    cap
                } else {
                    100
                },
                if resource == HtmlSemanticResource::ScratchRecords {
                    cap
                } else {
                    1000
                },
                0,
                if resource == HtmlSemanticResource::AllocatedBytes {
                    cap
                } else {
                    1000000
                },
                if resource == HtmlSemanticResource::NestedOutputRecords {
                    cap
                } else {
                    100000
                },
            );
            let mut ss = session(&c, &cx, l, 0, HtmlSemanticCancellation::new());
            let r = ss.query(std::slice::from_ref(&req));
            if cap == exact {
                assert!(r.is_ok(), "{resource:?}: {r:?}");
            } else {
                let failure = r.unwrap_err();
                assert!(
                    matches!(failure.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r==resource),
                    "{resource:?}: {failure:?}"
                );
                assert_eq!(failure.qualification().unwrap().capture(), c.identity());
                assert_eq!(ss.query(&[]).unwrap_err(), failure);
            }
            assert_eq!(ss.usage().scratch_records(), 0);
        }
    }
}

#[test]
fn late_batch_exhaustion_keeps_no_success_prefix_and_records_cumulative_output() {
    let (c, o) = leaf_option(&[], &["AB".into()]);
    let cx = context(&c, &[]);
    let req = request(&o, HtmlSemanticQuery::OptionText);
    let mut ss = session(
        &c,
        &cx,
        HtmlSemanticLimits::new(limits().work(), 2, 100, 0, 1000000, 2),
        0,
        HtmlSemanticCancellation::new(),
    );
    let fail = ss.query(&[req.clone(), req]).unwrap_err();
    assert!(matches!(
        fail.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::NestedOutputRecords,
            ..
        }
    ));
    assert_eq!(ss.usage().scratch_records(), 0);
    assert_eq!(fail.qualification().unwrap().context(), cx.identity());
    assert_eq!(ss.query(&[]).unwrap_err(), fail);
}

#[test]
fn cancellation_and_foreign_capture_rejection_are_sticky_and_qualified() {
    let (c, o) = leaf_option(&[], &["abc".into()]);
    let cx = context(&c, &[]);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut ss = session(&c, &cx, limits(), 0, cancel);
    let fail = ss
        .query(&[request(&o, HtmlSemanticQuery::OptionText)])
        .unwrap_err();
    assert_eq!(fail.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(fail.qualification().unwrap().capture(), c.identity());
    assert_eq!(ss.query(&[]).unwrap_err(), fail);
    let (_, other) = leaf_option(&[], &[]);
    let fail = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
        .query(&[request(&other, HtmlSemanticQuery::OptionValue)])
        .unwrap_err();
    assert!(matches!(fail.kind(), HtmlSemanticFailureKind::Capture(_)));
}

#[test]
fn state_only_context_change_recomputes_and_qualifies_select_value() {
    let (c, s, os) = simple_select();
    let a = context(&c, &[(os[0].clone(), Some(true))]);
    let b = context(
        &c,
        &[
            (os[0].clone(), Some(false)),
            (os[1].clone(), Some(false)),
            (os[2].clone(), Some(true)),
        ],
    );
    let req = request(&s, HtmlSemanticQuery::SelectValue);
    let ar = session(&c, &a, limits(), 100, HtmlSemanticCancellation::new())
        .query(std::slice::from_ref(&req))
        .unwrap();
    let br = session(&c, &b, limits(), 100, HtmlSemanticCancellation::new())
        .query(&[req])
        .unwrap();
    assert_eq!(
        select(ar.results()[0].fact()).value(),
        &DocumentString::from("first")
    );
    assert_eq!(
        select(br.results()[0].fact()).value(),
        &DocumentString::from("third")
    );
    assert_ne!(ar.context(), br.context());
    assert_eq!(ar.capture(), br.capture());
    assert_eq!(ar.host_revision(), c.host_facts_revision());
}

#[test]
fn select_value_consumes_exact_list_pruning_instead_of_generic_selected_descendants() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[]);
    let wrapper = html(&mut b, &d, "div", &[]);
    let a = html(&mut b, &d, "option", &[("value", "A")]);
    let excluded_nested_option = html(&mut b, &d, "option", &[("value", "wrong nested option")]);
    children(&mut b, &a, std::slice::from_ref(&excluded_nested_option));
    children(&mut b, &wrapper, std::slice::from_ref(&a));
    let mut barriers = Vec::new();
    let mut flags = vec![(excluded_nested_option, Some(true))];
    for name in ["hr", "datalist", "select"] {
        let barrier = html(&mut b, &d, name, &[]);
        let excluded = html(&mut b, &d, "option", &[("value", "wrong barrier")]);
        children(&mut b, &barrier, std::slice::from_ref(&excluded));
        barriers.push(barrier);
        flags.push((excluded, Some(true)));
    }
    let group = html(&mut b, &d, "optgroup", &[]);
    let foreign_group = element(&mut b, &d, Namespace::svg(), "optgroup", attrs(&[]));
    let nested_group = html(&mut b, &d, "optgroup", &[]);
    let excluded_group = html(&mut b, &d, "option", &[("value", "wrong second group")]);
    children(&mut b, &nested_group, std::slice::from_ref(&excluded_group));
    let f = html(&mut b, &d, "option", &[("value", "F")]);
    children(&mut b, &foreign_group, &[nested_group, f.clone()]);
    children(&mut b, &group, &[foreign_group]);
    let later_unknown = html(&mut b, &d, "div", &[]);
    let mut order = vec![wrapper];
    order.extend(barriers);
    order.extend([group, later_unknown]);
    children(&mut b, &s, &order);
    flags.extend([
        (excluded_group, Some(true)),
        (a, Some(false)),
        (f.clone(), Some(true)),
    ]);
    let c = b.finish().unwrap();
    let flags: Vec<_> = flags
        .into_iter()
        .map(|(h, v)| (c.subject_id(&h).unwrap(), v))
        .collect();
    let cx = context(&c, &flags);
    let fact = query(
        &c,
        &cx,
        &c.subject_id(&s).unwrap(),
        HtmlSemanticQuery::SelectValue,
    );
    assert_eq!(select(&fact).value(), &DocumentString::from("F"));
    assert_eq!(
        select(&fact).selected_option(),
        Some(&c.subject_id(&f).unwrap())
    );
}

#[test]
fn foreign_select_wrapper_is_traversed_and_empty_select_needs_no_option_state() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[]);
    let wrapper = element(&mut b, &d, Namespace::svg(), "select", attrs(&[]));
    let o = html(&mut b, &d, "option", &[("value", "foreign wrapper")]);
    children(&mut b, &wrapper, std::slice::from_ref(&o));
    children(&mut b, &s, &[wrapper]);
    let empty = html(&mut b, &d, "select", &[]);
    children(&mut b, &empty, &[]);
    let c = b.finish().unwrap();
    let cx = context(&c, &[(c.subject_id(&o).unwrap(), Some(true))]);
    assert_eq!(
        select(&query(
            &c,
            &cx,
            &c.subject_id(&s).unwrap(),
            HtmlSemanticQuery::SelectValue
        ))
        .value(),
        &DocumentString::from("foreign wrapper")
    );
    let l = HtmlSemanticLimits::new(limits().work(), 1, 0, 0, 100000, 0);
    let r = session(&c, &context(&c, &[]), l, 0, HtmlSemanticCancellation::new())
        .query(&[request(
            &c.subject_id(&empty).unwrap(),
            HtmlSemanticQuery::SelectValue,
        )])
        .unwrap();
    assert!(select(r.results()[0].fact()).value().is_empty());
    assert_eq!(select(r.results()[0].fact()).selected_option(), None);
    assert_eq!(r.usage().peak_scratch_records(), 0);
    assert_eq!(r.usage().nested_output_records(), 0);
}

#[test]
fn detached_option_and_ordinary_host_children_do_not_include_shadow_or_template_content() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let o = html(&mut b, &d, "option", &[]);
        b.establish_root(&o).unwrap();
        let host = html(&mut b, &d, if shadow { "div" } else { "template" }, &[]);
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
        let ignored = text(&mut b, &d, "wrong hosted text");
        children(&mut b, &fragment, &[ignored]);
        let ordinary = text(&mut b, &d, "ordinary");
        children(&mut b, &host, &[ordinary]);
        children(&mut b, &o, &[host]);
        let c = b.finish().unwrap();
        assert_eq!(
            option(&query(
                &c,
                &context(&c, &[]),
                &c.subject_id(&o).unwrap(),
                HtmlSemanticQuery::OptionText
            ))
            .value(),
            &DocumentString::from("ordinary")
        );
    }
}

#[test]
fn actual_script_and_image_text_is_excluded_while_processing_instructions_are_ignored() {
    let (mut b, d) = builder();
    let o = html(&mut b, &d, "option", &[]);
    let mut nodes = Vec::new();
    for (ns, name) in [
        (Namespace::html(), "script"),
        (Namespace::svg(), "script"),
        (Namespace::html(), "img"),
    ] {
        let e = element(&mut b, &d, ns, name, attrs(&[("alt", "wrong alt")]));
        let t = text(&mut b, &d, "wrong nested text");
        children(&mut b, &e, &[t]);
        nodes.push(e);
    }
    nodes.push(
        b.add_processing_instruction(&d, "p".into(), "wrong instruction".into())
            .unwrap(),
    );
    nodes.push(text(&mut b, &d, "right"));
    children(&mut b, &o, &nodes);
    let c = b.finish().unwrap();
    assert_eq!(
        option(&query(
            &c,
            &context(&c, &[]),
            &c.subject_id(&o).unwrap(),
            HtmlSemanticQuery::OptionText
        ))
        .value(),
        &DocumentString::from("right")
    );
}

#[test]
fn wrong_roles_case_and_foreign_subjects_do_not_require_child_or_state_facts() {
    let (mut b, d) = builder();
    let div = html(&mut b, &d, "div", &[]);
    let upper = html(&mut b, &d, "OPTION", &[]);
    let foreign = element(&mut b, &d, Namespace::svg(), "option", attrs(&[]));
    let text = text(&mut b, &d, "plain");
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    for (h, foreign) in [
        (div, false),
        (upper, false),
        (foreign, true),
        (text, false),
        (d, false),
    ] {
        let id = c.subject_id(&h).unwrap();
        for q in [
            HtmlSemanticQuery::OptionText,
            HtmlSemanticQuery::OptionLabel,
            HtmlSemanticQuery::OptionValue,
            HtmlSemanticQuery::SelectValue,
        ] {
            let f = query(&c, &cx, &id, q);
            match f {
                HtmlSemanticFact::OptionText(v)
                | HtmlSemanticFact::OptionLabel(v)
                | HtmlSemanticFact::OptionValue(v) => assert_eq!(
                    v,
                    if foreign {
                        HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
                    } else {
                        HtmlSemanticValue::Inapplicable
                    }
                ),
                HtmlSemanticFact::SelectValue(v) => assert_eq!(
                    v,
                    if foreign {
                        HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
                    } else {
                        HtmlSemanticValue::Inapplicable
                    }
                ),
                other => panic!("wrong fact {other:?}"),
            }
        }
    }
}

#[test]
fn deep_borrowed_walk_releases_scratch_after_success_missing_and_terminal_stop() {
    let (mut b, d) = builder();
    let o = html(&mut b, &d, "option", &[]);
    let ws: Vec<_> = (0..40).map(|_| html(&mut b, &d, "span", &[])).collect();
    children(&mut b, &o, std::slice::from_ref(&ws[0]));
    for pair in ws.windows(2) {
        children(&mut b, &pair[0], std::slice::from_ref(&pair[1]));
    }
    let t = text(&mut b, &d, "deep");
    children(&mut b, ws.last().unwrap(), &[t]);
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    let id = c.subject_id(&o).unwrap();
    let r = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
        .query(&[request(&id, HtmlSemanticQuery::OptionText)])
        .unwrap();
    assert_eq!(
        option(r.results()[0].fact()).value(),
        &DocumentString::from("deep")
    );
    assert!(r.usage().peak_scratch_records() >= 41);
    assert_eq!(r.usage().scratch_records(), 0);
    let mut stopped = session(
        &c,
        &cx,
        HtmlSemanticLimits::new(limits().work(), 1, 40, 0, 1000000, 1000),
        0,
        HtmlSemanticCancellation::new(),
    );
    assert!(
        stopped
            .query(&[request(&id, HtmlSemanticQuery::OptionText)])
            .is_err()
    );
    assert_eq!(stopped.usage().scratch_records(), 0);
}
