use crate::semantics::support::{
    builder, collection_limits as resource_limits, context, semantic_limits as limits, session,
};
use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlCapturedShadowProperties,
    HtmlSemanticCancellation, HtmlSemanticContext, HtmlSemanticDependencies, HtmlSemanticFact,
    HtmlSemanticFailureKind, HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport,
    HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticUnsupported, HtmlSemanticValue,
};

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

fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::SelectPlaceholderLabelOption)
}
fn report(
    c: &HtmlCapture,
    cx: &HtmlSemanticContext,
    id: &HtmlCaptureSubjectId,
    raw: usize,
) -> HtmlSemanticReport {
    session(c, cx, limits(), raw, HtmlSemanticCancellation::new())
        .query(&[request(id)])
        .unwrap()
}
fn known(r: &HtmlSemanticReport) -> Option<&HtmlCaptureSubjectId> {
    match r.results()[0].fact() {
        HtmlSemanticFact::SelectPlaceholderLabelOption(HtmlSemanticValue::Known(v)) => v.as_ref(),
        other => panic!("expected known placeholder, got {other:?}"),
    }
}
fn missing(r: &HtmlSemanticReport) -> &HtmlSemanticMissing {
    match r.results()[0].fact() {
        HtmlSemanticFact::SelectPlaceholderLabelOption(HtmlSemanticValue::Missing(v)) => v,
        other => panic!("expected missing placeholder, got {other:?}"),
    }
}
fn fixture(
    attrs: &[(&str, &str)],
    option_attrs: &[(&str, &str)],
    text: Option<&str>,
) -> (HtmlCapture, HtmlCaptureSubjectId, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", attrs);
    let o = html(&mut b, &d, "option", option_attrs);
    if let Some(text) = text {
        let t = b.add_text(&d, text.into()).unwrap();
        children(&mut b, &o, &[t]);
    }
    children(&mut b, &s, std::slice::from_ref(&o));
    let c = b.finish().unwrap();
    let sid = c.subject_id(&s).unwrap();
    let oid = c.subject_id(&o).unwrap();
    (c, sid, oid)
}

#[test]
fn recovered_display_size_and_conditional_multiple_default_select_the_first_empty_option() {
    // Source processing allows prefixes/plus/-zero; strict author spelling is separate.
    for (size, multiple, expected) in [
        (None, false, true),
        (None, true, false),
        (Some(""), false, true),
        (Some("nope"), true, false),
        (Some("0"), false, false),
        (Some("2"), false, false),
        (Some("1"), false, true),
        (Some("1"), true, true),
        (Some("+1"), false, true),
        (Some(" \t1tail"), false, true),
        (Some("01"), false, true),
        (Some("-0"), false, false),
        (Some("-1"), false, true),
        (
            Some("99999999999999999999999999999999999999999999"),
            false,
            false,
        ),
        (Some("1e999"), false, true),
        (Some("1.5"), true, true),
        (Some("\u{a0}1"), true, false),
        (Some("-000"), true, false),
    ] {
        let mut attrs = vec![("required", "not a conforming Boolean spelling")];
        if let Some(size) = size {
            attrs.push(("size", size));
        }
        if multiple {
            attrs.push(("multiple", "false"));
        }
        let (c, s, o) = fixture(&attrs, &[("value", "")], None);
        let r = report(&c, &context(&c), &s, 100);
        assert_eq!(
            known(&r),
            expected.then_some(&o),
            "size={size:?}, multiple={multiple}"
        );
        assert_eq!(r.usage().scratch_records(), 0);
    }
}

#[test]
fn absent_required_bypasses_oversized_size_and_unknown_children() {
    let (mut b, d) = builder();
    let s = html(
        &mut b,
        &d,
        "select",
        &[("size", "a very long unused value"), ("multiple", "unused")],
    );
    let c = b.finish().unwrap();
    let r = report(&c, &context(&c), &c.subject_id(&s).unwrap(), 0);
    assert_eq!(known(&r), None);
    assert_eq!(r.usage().peak_scratch_records(), 0);
    assert_eq!(r.usage().nested_output_records(), 0);
}

#[test]
fn boolean_presence_does_not_admit_unused_spelling_and_successful_size_skips_multiple() {
    for attrs in [
        vec![("required", "enormous invalid spelling")],
        vec![
            ("required", "enormous invalid spelling"),
            ("multiple", "enormous invalid spelling"),
            ("size", "1"),
        ],
    ] {
        let (c, s, o) = fixture(&attrs, &[("value", "")], None);
        assert_eq!(
            known(&report(&c, &context(&c), &s, usize::from(attrs.len() > 1))),
            Some(&o)
        );
    }
    let (c, s, _) = fixture(
        &[("required", "enormous"), ("multiple", "enormous")],
        &[("value", "oversized")],
        None,
    );
    assert_eq!(known(&report(&c, &context(&c), &s, 0)), None);
}

#[test]
fn successful_zero_or_large_size_bypasses_unavailable_order_and_unused_multiple() {
    for size in ["0", "2", "999999999999999999999999999999999999999"] {
        let (mut b, d) = builder();
        let s = html(
            &mut b,
            &d,
            "select",
            &[("required", ""), ("size", size), ("multiple", "oversized")],
        );
        let c = b.finish().unwrap();
        let r = report(&c, &context(&c), &c.subject_id(&s).unwrap(), size.len());
        assert_eq!(known(&r), None);
        assert_eq!(r.usage().peak_scratch_records(), 0);
    }
}

#[test]
fn original_size_prefix_and_value_content_have_exact_raw_admission() {
    let (c, s, o) = fixture(
        &[("required", ""), ("size", " \t+1tail")],
        &[("value", "")],
        None,
    );
    let cx = context(&c);
    assert_eq!(known(&report(&c, &cx, &s, 8)), Some(&o));
    let f = session(&c, &cx, limits(), 7, HtmlSemanticCancellation::new())
        .query(&[request(&s)])
        .unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
    let (c, s, _) = fixture(&[("required", "")], &[("value", "ab")], None);
    let cx = context(&c);
    assert_eq!(known(&report(&c, &cx, &s, 2)), None);
    let f = session(&c, &cx, limits(), 1, HtmlSemanticCancellation::new())
        .query(&[request(&s)])
        .unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
}

#[test]
fn namespaced_required_is_absent_and_namespaced_value_does_not_override_text() {
    let (mut b, d) = builder();
    let s = element(
        &mut b,
        &d,
        Namespace::html(),
        "select",
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::svg(), None, "required"),
            "",
        )])
        .unwrap(),
    );
    let c = b.finish().unwrap();
    assert_eq!(
        known(&report(&c, &context(&c), &c.subject_id(&s).unwrap(), 0)),
        None
    );
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let o = element(
        &mut b,
        &d,
        Namespace::html(),
        "option",
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::svg(), None, "value"),
            "ignored",
        )])
        .unwrap(),
    );
    children(&mut b, &o, &[]);
    children(&mut b, &s, std::slice::from_ref(&o));
    let c = b.finish().unwrap();
    assert_eq!(
        known(&report(&c, &context(&c), &c.subject_id(&s).unwrap(), 0)),
        Some(&c.subject_id(&o).unwrap())
    );
}

#[test]
fn required_size_one_with_unknown_option_order_is_missing_not_none() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&s).unwrap();
    assert_eq!(
        missing(&report(&c, &context(&c), &id, 0)),
        &HtmlSemanticMissing::OrderedChildren(id)
    );
}

#[test]
fn incomplete_prefix_influences_first_option_but_later_unknown_subtree_does_not() {
    for prefix in [true, false] {
        let (mut b, d) = builder();
        let s = html(&mut b, &d, "select", &[("required", "")]);
        let w = html(&mut b, &d, "div", &[]);
        let o = html(&mut b, &d, "option", &[("value", "")]);
        let ordered = if prefix {
            [w.clone(), o.clone()]
        } else {
            [o.clone(), w.clone()]
        };
        children(&mut b, &s, &ordered);
        let c = b.finish().unwrap();
        let r = report(&c, &context(&c), &c.subject_id(&s).unwrap(), 0);
        if prefix {
            assert_eq!(
                missing(&r),
                &HtmlSemanticMissing::OrderedChildren(c.subject_id(&w).unwrap())
            );
        } else {
            assert_eq!(known(&r), Some(&c.subject_id(&o).unwrap()));
        }
        assert_eq!(r.usage().scratch_records(), 0);
    }
}

#[test]
fn first_nonempty_option_is_not_replaced_by_later_empty_option() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let first = html(&mut b, &d, "option", &[("value", "nonempty")]);
    let later = html(&mut b, &d, "option", &[("value", "")]);
    children(&mut b, &s, &[first, later]);
    let c = b.finish().unwrap();
    assert_eq!(
        known(&report(&c, &context(&c), &c.subject_id(&s).unwrap(), 8)),
        None
    );
}

#[test]
fn optgroup_exclusion_bypasses_oversized_value_and_unknown_text_without_searching_later_options() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let g = html(&mut b, &d, "optgroup", &[]);
    let w = html(&mut b, &d, "div", &[]);
    let o = html(&mut b, &d, "option", &[("value", "oversized")]);
    let later = html(&mut b, &d, "option", &[("value", "")]);
    children(&mut b, &w, &[o]);
    children(&mut b, &g, &[w]);
    children(&mut b, &s, &[g, later]);
    let c = b.finish().unwrap();
    let r = report(&c, &context(&c), &c.subject_id(&s).unwrap(), 0);
    assert_eq!(known(&r), None);
    assert_eq!(r.usage().nested_output_records(), 0);
}

#[test]
fn ordinary_wrappers_allow_first_option_and_barriers_prune_their_whole_subtrees() {
    for barrier in ["select", "hr", "datalist", "second-optgroup"] {
        let (mut b, d) = builder();
        let s = html(&mut b, &d, "select", &[("required", "")]);
        let hidden = html(&mut b, &d, "option", &[("value", "not consulted")]);
        let p = html(
            &mut b,
            &d,
            if barrier == "second-optgroup" {
                "optgroup"
            } else {
                barrier
            },
            &[],
        );
        children(&mut b, &p, &[hidden]);
        let p = if barrier == "second-optgroup" {
            let outer = html(&mut b, &d, "optgroup", &[]);
            children(&mut b, &outer, &[p]);
            outer
        } else {
            p
        };
        let o = html(&mut b, &d, "option", &[("value", "")]);
        let w = html(&mut b, &d, "div", &[]);
        children(&mut b, &w, std::slice::from_ref(&o));
        children(&mut b, &s, &[p, w]);
        let c = b.finish().unwrap();
        assert_eq!(
            known(&report(&c, &context(&c), &c.subject_id(&s).unwrap(), 0)),
            Some(&c.subject_id(&o).unwrap()),
            "{barrier}"
        );
    }
}

#[test]
fn foreign_and_uppercase_optgroup_lookalikes_do_not_disqualify_the_option() {
    for (ns, name) in [
        (Namespace::svg(), "optgroup"),
        (Namespace::html(), "OPTGROUP"),
    ] {
        let (mut b, d) = builder();
        let s = html(&mut b, &d, "select", &[("required", "")]);
        let w = element(&mut b, &d, ns, name, OrderedAttributes::new([]).unwrap());
        let o = html(&mut b, &d, "option", &[("value", "")]);
        children(&mut b, &w, std::slice::from_ref(&o));
        children(&mut b, &s, &[w]);
        let c = b.finish().unwrap();
        assert_eq!(
            known(&report(&c, &context(&c), &c.subject_id(&s).unwrap(), 0)),
            Some(&c.subject_id(&o).unwrap())
        );
    }
}

#[test]
fn complete_empty_options_proves_none_without_scratch_or_state() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    children(&mut b, &s, &[]);
    let c = b.finish().unwrap();
    let r = report(&c, &context(&c), &c.subject_id(&s).unwrap(), 0);
    assert_eq!(known(&r), None);
    assert_eq!(r.usage().peak_scratch_records(), 0);
    assert_eq!(r.usage().nested_output_records(), 0);
}

#[test]
fn value_attribute_including_empty_wins_over_unknown_children_and_label() {
    let (c, s, o) = fixture(
        &[("required", "")],
        &[
            ("value", ""),
            ("label", "unused enormous label"),
            ("selected", "false"),
            ("disabled", ""),
        ],
        None,
    );
    assert_eq!(known(&report(&c, &context(&c), &s, 0)), Some(&o));
    let (c, s, _) = fixture(&[("required", "")], &[("label", "")], None);
    let r = report(&c, &context(&c), &s, 0);
    assert!(matches!(
        missing(&r),
        HtmlSemanticMissing::OrderedChildren(_)
    ));
}

#[test]
fn normalized_option_text_uses_only_ascii_whitespace_for_emptiness() {
    for (text, empty) in [
        ("", true),
        (" \t\r\n\u{c}", true),
        ("\u{a0}", false),
        (" \tvisible ", false),
    ] {
        let (c, s, o) = fixture(&[("required", "")], &[], Some(text));
        assert_eq!(
            known(&report(&c, &context(&c), &s, 100)),
            empty.then_some(&o),
            "{text:?}"
        );
    }
}

#[test]
fn option_text_excludes_script_img_subtrees_and_has_no_br_separator() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let o = html(&mut b, &d, "option", &[]);
    let mut nodes = vec![];
    for (ns, name) in [
        (Namespace::html(), "script"),
        (Namespace::svg(), "script"),
        (Namespace::html(), "img"),
    ] {
        let w = element(&mut b, &d, ns, name, OrderedAttributes::new([]).unwrap());
        let t = b.add_text(&d, "excluded".into()).unwrap();
        children(&mut b, &w, &[t]);
        nodes.push(w);
    }
    let br = html(&mut b, &d, "br", &[]);
    children(&mut b, &br, &[]);
    nodes.push(br);
    children(&mut b, &o, &nodes);
    children(&mut b, &s, std::slice::from_ref(&o));
    let c = b.finish().unwrap();
    assert_eq!(
        known(&report(&c, &context(&c), &c.subject_id(&s).unwrap(), 0)),
        Some(&c.subject_id(&o).unwrap())
    );
}

#[test]
fn surrogate_value_and_contributing_text_remain_nonempty_and_admitted_as_utf16_units() {
    for attribute in [true, false] {
        let (mut b, d) = builder();
        let s = html(&mut b, &d, "select", &[("required", "")]);
        let raw = DocumentString::from_utf16(vec![0xd800, 0xdc00, 0xdfff]);
        let o = element(
            &mut b,
            &d,
            Namespace::html(),
            "option",
            if attribute {
                OrderedAttributes::new([Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "value"),
                    raw,
                )])
                .unwrap()
            } else {
                OrderedAttributes::new([]).unwrap()
            },
        );
        if !attribute {
            let t = b
                .add_text(&d, DocumentString::from_utf16(vec![0xd800, 0xdc00, 0xdfff]))
                .unwrap();
            children(&mut b, &o, &[t]);
        }
        children(&mut b, &s, &[o]);
        let c = b.finish().unwrap();
        let s = c.subject_id(&s).unwrap();
        let cx = context(&c);
        assert_eq!(known(&report(&c, &cx, &s, 3)), None);
        if attribute {
            let f = session(&c, &cx, limits(), 2, HtmlSemanticCancellation::new())
                .query(&[request(&s)])
                .unwrap_err();
            assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
        } else {
            // Descendant text has semantic work/vector budgets, not the
            // attribute-only input cap. Its three exact units still require
            // actual output capacity when this ID projection returns None.
            let r = report(&c, &cx, &s, 0);
            assert_eq!(known(&r), None);
            assert!(r.usage().nested_output_records() >= 3);
            let f = session(
                &c,
                &cx,
                resource_limits(HtmlSemanticResource::NestedOutputRecords, 2),
                0,
                HtmlSemanticCancellation::new(),
            )
            .query(&[request(&s)])
            .unwrap_err();
            assert!(matches!(
                f.kind(),
                HtmlSemanticFailureKind::LimitExceeded {
                    resource: HtmlSemanticResource::NestedOutputRecords,
                    ..
                }
            ));
        }
    }
}

#[test]
fn non_select_subjects_and_namespace_lookalikes_have_typed_applicability() {
    let (mut b, d) = builder();
    let upper = html(&mut b, &d, "SELECT", &[("required", "")]);
    let option = html(&mut b, &d, "option", &[]);
    let foreign = element(
        &mut b,
        &d,
        Namespace::svg(),
        "select",
        OrderedAttributes::new([]).unwrap(),
    );
    let text = b.add_text(&d, "text".into()).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    for h in [upper, option, text, d] {
        assert_eq!(
            report(&c, &cx, &c.subject_id(&h).unwrap(), 0).results()[0].fact(),
            &HtmlSemanticFact::SelectPlaceholderLabelOption(HtmlSemanticValue::Inapplicable)
        );
    }
    assert_eq!(
        report(&c, &cx, &c.subject_id(&foreign).unwrap(), 0).results()[0].fact(),
        &HtmlSemanticFact::SelectPlaceholderLabelOption(HtmlSemanticValue::Unsupported(
            HtmlSemanticUnsupported::ForeignNamespace
        ))
    );
}

#[test]
fn text_projection_has_exact_one_short_work_output_scratch_capacity_and_byte_boundaries() {
    let (c, s, _) = fixture(
        &[("required", ""), ("size", "1")],
        &[],
        Some("nonempty normalized text"),
    );
    let cx = context(&c);
    let baseline = report(&c, &cx, &s, 100);
    assert_eq!(known(&baseline), None);
    let u = baseline.usage();
    assert_eq!(u.scratch_records(), 0);
    for (resource, exact) in [
        (HtmlSemanticResource::Work, u.work()),
        (HtmlSemanticResource::OutputRecords, u.output_records()),
        (
            HtmlSemanticResource::ScratchRecords,
            u.peak_scratch_records(),
        ),
        (HtmlSemanticResource::AllocatedBytes, u.allocated_bytes()),
        (
            HtmlSemanticResource::NestedOutputRecords,
            u.nested_output_records(),
        ),
    ] {
        assert!(exact > 0, "{resource:?}");
        for cap in [exact, exact - 1] {
            let mut ss = session(
                &c,
                &cx,
                resource_limits(resource, cap),
                100,
                HtmlSemanticCancellation::new(),
            );
            let r = ss.query(&[request(&s)]);
            if cap == exact {
                assert_eq!(known(&r.unwrap()), None);
            } else {
                let f = r.unwrap_err();
                assert!(
                    matches!(f.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r==resource),
                    "{f:?}"
                );
                assert_eq!(f.qualification().unwrap().context(), cx.identity());
                assert_eq!(ss.query(&[]).unwrap_err(), f);
            }
            assert_eq!(ss.usage().scratch_records(), 0);
        }
    }
}

#[test]
fn discarded_text_intermediates_remain_cumulatively_admitted_across_duplicate_queries() {
    let (c, s, _) = fixture(&[("required", "")], &[], Some("not empty"));
    let cx = context(&c);
    let single = report(&c, &cx, &s, 100).usage();
    let mut ss = session(&c, &cx, limits(), 100, HtmlSemanticCancellation::new());
    let r = ss.query(&[request(&s), request(&s)]).unwrap();
    assert_eq!(
        r.usage().nested_output_records(),
        2 * single.nested_output_records()
    );
    assert!(r.usage().allocated_bytes() > single.allocated_bytes());
    assert_eq!(r.usage().scratch_records(), 0);
    let mut ss = session(
        &c,
        &cx,
        resource_limits(
            HtmlSemanticResource::NestedOutputRecords,
            single.nested_output_records(),
        ),
        100,
        HtmlSemanticCancellation::new(),
    );
    let f = ss.query(&[request(&s), request(&s)]).unwrap_err();
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::NestedOutputRecords,
            ..
        }
    ));
    assert_eq!(ss.query(&[]).unwrap_err(), f);
    assert_eq!(ss.usage().scratch_records(), 0);
}

#[test]
fn late_content_failure_and_cancel_are_sticky_qualified_atomic_batch_stops() {
    let (mut b, d) = builder();
    let a = html(&mut b, &d, "select", &[]);
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let o = html(&mut b, &d, "option", &[("value", "long")]);
    children(&mut b, &s, &[o]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let a = c.subject_id(&a).unwrap();
    let s = c.subject_id(&s).unwrap();
    let mut ss = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let f = ss.query(&[request(&a), request(&s)]).unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(f.qualification().unwrap().capture(), c.identity());
    assert_eq!(ss.query(&[]).unwrap_err(), f);
    assert_eq!(ss.usage().scratch_records(), 0);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut ss = session(&c, &cx, limits(), 100, cancel);
    let f = ss.query(&[request(&s)]).unwrap_err();
    assert_eq!(f.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(ss.query(&[]).unwrap_err(), f);
}

#[test]
fn independent_capture_attribute_and_text_changes_recompute_exact_qualification() {
    let (a, s, _) = fixture(&[("required", "")], &[], Some(""));
    let (b, t, _) = fixture(&[("required", "")], &[], Some("changed"));
    let ax = context(&a);
    let bx = context(&b);
    let ar = report(&a, &ax, &s, 100);
    let br = report(&b, &bx, &t, 100);
    assert!(known(&ar).is_some());
    assert_eq!(known(&br), None);
    assert_ne!(ar.capture(), br.capture());
    assert_ne!(ar.context(), br.context());
    assert_eq!(
        ar.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
    let f = session(&a, &ax, limits(), 100, HtmlSemanticCancellation::new())
        .query(&[request(&t)])
        .unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Capture(_)));
}

#[test]
fn shadow_and_template_content_options_are_not_ordinary_select_descendants() {
    for shadow in [true, false] {
        let (mut b, d) = builder();
        let s = html(&mut b, &d, "select", &[("required", "")]);
        let host = if shadow {
            html(&mut b, &d, "div", &[])
        } else {
            html(&mut b, &d, "template", &[])
        };
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
        let o = html(&mut b, &d, "option", &[("value", "")]);
        children(&mut b, &fragment, &[o]);
        children(&mut b, &host, &[]);
        children(&mut b, &s, &[host]);
        let c = b.finish().unwrap();
        assert_eq!(
            known(&report(&c, &context(&c), &c.subject_id(&s).unwrap(), 0)),
            None
        );
    }
}

#[test]
fn literal_attribute_case_and_namespace_do_not_create_boolean_or_size_conditions() {
    for attrs in [
        [("Required", ""), ("size", "1")],
        [("required", ""), ("Size", "2")],
        [("required", ""), ("Multiple", "")],
    ] {
        let (c, s, o) = fixture(&attrs, &[("value", "")], None);
        assert_eq!(
            known(&report(&c, &context(&c), &s, 1)),
            if attrs[0].0 == "required" {
                Some(&o)
            } else {
                None
            }
        );
    }
    let (mut b, d) = builder();
    let s = element(
        &mut b,
        &d,
        Namespace::html(),
        "select",
        OrderedAttributes::new([
            Attribute::new(QualifiedName::new(Namespace::none(), None, "required"), ""),
            Attribute::new(
                QualifiedName::new(Namespace::svg(), None, "size"),
                "oversized",
            ),
            Attribute::new(
                QualifiedName::new(Namespace::svg(), None, "multiple"),
                "oversized",
            ),
        ])
        .unwrap(),
    );
    let o = html(&mut b, &d, "option", &[("value", "")]);
    children(&mut b, &s, std::slice::from_ref(&o));
    let c = b.finish().unwrap();
    assert_eq!(
        known(&report(&c, &context(&c), &c.subject_id(&s).unwrap(), 0)),
        Some(&c.subject_id(&o).unwrap())
    );
}

#[test]
fn foreign_and_uppercase_option_lookalikes_are_not_the_first_source_option() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let svg = element(
        &mut b,
        &d,
        Namespace::svg(),
        "option",
        OrderedAttributes::new([]).unwrap(),
    );
    let upper = html(&mut b, &d, "OPTION", &[("value", "unused oversized")]);
    children(&mut b, &svg, &[]);
    children(&mut b, &upper, &[]);
    let o = html(&mut b, &d, "option", &[("value", "")]);
    children(&mut b, &s, &[svg, upper, o.clone()]);
    let c = b.finish().unwrap();
    assert_eq!(
        known(&report(&c, &context(&c), &c.subject_id(&s).unwrap(), 0)),
        Some(&c.subject_id(&o).unwrap())
    );
}
