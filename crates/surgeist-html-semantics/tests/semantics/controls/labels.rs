use crate::semantics::support::{
    compact_builder_without_registries as builder, moderate_semantic_limits_with_scratch as limits,
};
use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlCapturedNodeKind, HtmlCapturedShadowProperties,
    HtmlControlLabels, HtmlCustomControlClassification, HtmlSemanticCancellation,
    HtmlSemanticContext, HtmlSemanticContextInput, HtmlSemanticDependencies, HtmlSemanticFact,
    HtmlSemanticFailureKind, HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery,
    HtmlSemanticReport, HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticSession,
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
fn expanded(
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
fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    local: &str,
    attributes: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    expanded(b, d, Namespace::html(), local, attrs(attributes))
}
struct Fixture {
    capture: HtmlCapture,
    ids: Vec<HtmlCaptureSubjectId>,
}
fn finish(b: HtmlCaptureBuilder, handles: &[HtmlCaptureBuilderSubject]) -> Fixture {
    let capture = b.finish().unwrap();
    let ids = handles
        .iter()
        .map(|h| capture.subject_id(h).unwrap())
        .collect();
    Fixture { capture, ids }
}
fn flat(specs: &[(&str, &[(&str, &str)])], order: &[usize]) -> Fixture {
    let (mut b, d) = builder();
    let h: Vec<_> = specs
        .iter()
        .map(|(n, a)| element(&mut b, &d, n, a))
        .collect();
    for v in &h {
        b.establish_children(v, &[]).unwrap();
    }
    b.establish_children(&d, &order.iter().map(|i| h[*i].clone()).collect::<Vec<_>>())
        .unwrap();
    finish(b, &h)
}
fn context(f: &Fixture, classifications: &[(usize, bool)]) -> HtmlSemanticContext {
    HtmlSemanticContext::new(
        &f.capture,
        f.capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(
            classifications
                .iter()
                .map(|(i, v)| HtmlCustomControlClassification::new(f.ids[*i].clone(), *v))
                .collect(),
        ),
        100,
    )
    .unwrap()
}
fn session<'a>(
    f: &'a Fixture,
    c: &'a HtmlSemanticContext,
    l: HtmlSemanticLimits,
    raw: usize,
    cancel: HtmlSemanticCancellation,
) -> HtmlSemanticSession<'a> {
    HtmlSemanticSession::new(&f.capture, c, HtmlValueInterpreter::new(raw), l, cancel).unwrap()
}
fn report(f: &Fixture, c: &HtmlSemanticContext) -> HtmlSemanticReport {
    session(f, c, limits(), 10000, HtmlSemanticCancellation::new())
        .query(&[HtmlSemanticRequest::new(
            f.ids[0].clone(),
            HtmlSemanticQuery::ControlLabels,
        )])
        .unwrap()
}
fn value(r: &HtmlSemanticReport) -> &HtmlSemanticValue<Option<HtmlControlLabels>> {
    match r.results()[0].fact() {
        HtmlSemanticFact::ControlLabels(v) => v,
        other => panic!("unexpected {other:?}"),
    }
}
fn labels(r: &HtmlSemanticReport) -> &[HtmlCaptureSubjectId] {
    match value(r) {
        HtmlSemanticValue::Known(Some(v)) => v.labels(),
        other => panic!("expected complete labels {other:?}"),
    }
}

#[test]
fn explicit_labels_before_and_after_control_preserve_tree_order() {
    let f = flat(
        &[
            ("input", &[("id", "f")]),
            ("label", &[("for", "f")]),
            ("label", &[("for", "f")]),
            ("label", &[("for", "other")]),
        ],
        &[2, 0, 3, 1],
    );
    let c = context(&f, &[]);
    let r = report(&f, &c);
    assert_eq!(labels(&r), &[f.ids[2].clone(), f.ids[1].clone()]);
    assert_eq!(
        r.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
}
#[test]
fn all_native_labelable_roles_require_no_form_owner_or_current_state() {
    for local in [
        "button", "input", "meter", "output", "progress", "select", "textarea",
    ] {
        let f = flat(
            &[
                (local, &[("id", "f"), ("disabled", "false")]),
                ("label", &[("for", "f")]),
            ],
            &[1, 0],
        );
        let c = context(&f, &[]);
        assert_eq!(
            labels(&report(&f, &c)),
            std::slice::from_ref(&f.ids[1]),
            "{local}"
        );
    }
}
#[test]
fn hidden_null_needs_no_topology_and_missing_invalid_type_defaults_remain_labelable() {
    let (mut b, d) = builder();
    let input = element(&mut b, &d, "input", &[("type", "hidden")]);
    let f = finish(b, &[input]);
    let c = context(&f, &[]);
    assert_eq!(value(&report(&f, &c)), &HtmlSemanticValue::Known(None));
    for attributes in [&[][..], &[("type", "")][..], &[("type", "bad")][..]] {
        let (mut b, d) = builder();
        let input = element(&mut b, &d, "input", attributes);
        let f = finish(b, &[input]);
        let c = context(&f, &[]);
        assert_eq!(
            value(&report(&f, &c)),
            &HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(f.ids[0].clone()))
        );
    }
}
#[test]
fn complete_empty_list_is_distinct_from_hidden_null_and_unavailable_children() {
    let f = flat(&[("input", &[])], &[0]);
    let c = context(&f, &[]);
    assert!(labels(&report(&f, &c)).is_empty());
    let (mut b, d) = builder();
    let input = element(&mut b, &d, "input", &[]);
    b.establish_children(&d, std::slice::from_ref(&input))
        .unwrap();
    let f = finish(b, &[input]);
    let c = context(&f, &[]);
    assert_eq!(
        value(&report(&f, &c)),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(f.ids[0].clone()))
    );
}
#[test]
fn implicit_nested_labels_use_first_labelable_descendant() {
    for other_first in [false, true] {
        let (mut b, d) = builder();
        let target = element(&mut b, &d, "input", &[]);
        let outer = element(&mut b, &d, "label", &[]);
        let inner = element(&mut b, &d, "label", &[]);
        let other = element(&mut b, &d, "textarea", &[]);
        for h in [&target, &other] {
            b.establish_children(h, &[]).unwrap();
        }
        b.establish_children(&inner, std::slice::from_ref(&target))
            .unwrap();
        let order = if other_first {
            vec![other, inner.clone()]
        } else {
            vec![inner.clone(), other]
        };
        b.establish_children(&outer, &order).unwrap();
        b.establish_children(&d, std::slice::from_ref(&outer))
            .unwrap();
        let f = finish(b, &[target, outer, inner]);
        let c = context(&f, &[]);
        let r = report(&f, &c);
        let expected = if other_first {
            vec![f.ids[2].clone()]
        } else {
            vec![f.ids[1].clone(), f.ids[2].clone()]
        };
        assert_eq!(labels(&r), expected);
    }
}
#[test]
fn present_for_overrides_implicit_descendant_including_empty() {
    for target_for in ["other", ""] {
        let (mut b, d) = builder();
        let target = element(&mut b, &d, "input", &[("id", "target")]);
        let label = element(&mut b, &d, "label", &[("for", target_for)]);
        let other = element(&mut b, &d, "input", &[("id", "other")]);
        for h in [&target, &other] {
            b.establish_children(h, &[]).unwrap();
        }
        b.establish_children(&label, std::slice::from_ref(&target))
            .unwrap();
        b.establish_children(&d, &[label.clone(), other]).unwrap();
        let f = finish(b, &[target, label]);
        let c = context(&f, &[]);
        assert!(labels(&report(&f, &c)).is_empty());
    }
}
#[test]
fn duplicate_first_other_id_excludes_target_without_unknown_custom_labelability() {
    for first in ["div", "input", "x-unknown"] {
        let f = flat(
            &[
                ("input", &[("id", "f")]),
                (first, &[("id", "f"), ("type", "hidden")]),
                ("label", &[("for", "f")]),
            ],
            &[1, 2, 0],
        );
        let c = context(&f, &[]);
        assert!(labels(&report(&f, &c)).is_empty(), "{first}");
        if first == "x-unknown" {
            let r = session(&f, &c, limits(), 100, HtmlSemanticCancellation::new())
                .query(&[HtmlSemanticRequest::new(
                    f.ids[2].clone(),
                    HtmlSemanticQuery::LabeledControl,
                )])
                .unwrap();
            assert_eq!(
                r.results()[0].fact(),
                &HtmlSemanticFact::LabeledControl(HtmlSemanticValue::Missing(
                    HtmlSemanticMissing::CustomControlClassification(f.ids[1].clone())
                ))
            );
        }
    }
}
#[test]
fn unrelated_explicit_and_implicit_labels_do_not_need_custom_facts() {
    let (mut b, d) = builder();
    let target = element(&mut b, &d, "input", &[("id", "target")]);
    let explicit = element(&mut b, &d, "label", &[("for", "other")]);
    let implicit = element(&mut b, &d, "label", &[]);
    let custom = element(&mut b, &d, "x-unknown", &[("id", "other")]);
    for h in [&target, &custom, &explicit] {
        b.establish_children(h, &[]).unwrap();
    }
    b.establish_children(&implicit, std::slice::from_ref(&custom))
        .unwrap();
    b.establish_children(&d, &[explicit.clone(), implicit.clone(), target.clone()])
        .unwrap();
    let f = finish(b, &[target, explicit, implicit]);
    let c = context(&f, &[]);
    assert!(labels(&report(&f, &c)).is_empty());
}
#[test]
fn custom_target_true_false_and_unavailable_classifications_remain_distinct() {
    let f = flat(
        &[("x-control", &[("id", "f")]), ("label", &[("for", "f")])],
        &[1, 0],
    );
    let c = context(&f, &[]);
    assert_eq!(
        value(&report(&f, &c)),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::CustomControlClassification(
            f.ids[0].clone()
        ))
    );
    let positive = context(&f, &[(0, true)]);
    let yes = report(&f, &positive);
    assert_eq!(labels(&yes), std::slice::from_ref(&f.ids[1]));
    let negative = context(&f, &[(0, false)]);
    let no = report(&f, &negative);
    assert_eq!(value(&no), &HtmlSemanticValue::Inapplicable);
    assert_ne!(yes.qualification().context(), no.qualification().context());
    assert_eq!(yes.capture(), no.capture());
}
#[test]
fn earlier_unknown_implicit_candidate_is_influencing_context_and_no_partial_list_escapes() {
    let (mut b, d) = builder();
    let target = element(&mut b, &d, "input", &[("id", "f")]);
    let explicit = element(&mut b, &d, "label", &[("for", "f")]);
    let implicit = element(&mut b, &d, "label", &[]);
    let custom = element(&mut b, &d, "x-unknown", &[]);
    for h in [&target, &custom, &explicit] {
        b.establish_children(h, &[]).unwrap();
    }
    b.establish_children(&implicit, &[custom.clone(), target.clone()])
        .unwrap();
    b.establish_children(&d, &[explicit.clone(), implicit.clone()])
        .unwrap();
    let f = finish(b, &[target, explicit, implicit, custom]);
    let c = context(&f, &[]);
    assert_eq!(
        value(&report(&f, &c)),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::CustomControlClassification(
            f.ids[3].clone()
        ))
    );
    let c = context(&f, &[(3, false)]);
    assert_eq!(
        labels(&report(&f, &c)),
        &[f.ids[1].clone(), f.ids[2].clone()]
    );
    let c = context(&f, &[(3, true)]);
    assert_eq!(labels(&report(&f, &c)), std::slice::from_ref(&f.ids[1]));
}
#[test]
fn detached_ordinary_root_supports_implicit_labels_without_connectedness() {
    let (mut b, d) = builder();
    let target = element(&mut b, &d, "input", &[]);
    let label = element(&mut b, &d, "label", &[]);
    b.establish_root(&label).unwrap();
    b.establish_children(&target, &[]).unwrap();
    b.establish_children(&label, std::slice::from_ref(&target))
        .unwrap();
    let f = finish(b, &[target, label]);
    let c = context(&f, &[]);
    assert_eq!(labels(&report(&f, &c)), std::slice::from_ref(&f.ids[1]));
}
#[test]
fn shadow_and_template_hosts_do_not_supply_ordinary_labels() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let label = element(&mut b, &d, "label", &[("for", "f")]);
        let host = element(&mut b, &d, if shadow { "div" } else { "template" }, &[]);
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
        let target = element(&mut b, &d, "input", &[("id", "f")]);
        b.establish_children(&target, &[]).unwrap();
        b.establish_children(&fragment, std::slice::from_ref(&target))
            .unwrap();
        b.establish_children(&host, &[]).unwrap();
        b.establish_children(&label, std::slice::from_ref(&host))
            .unwrap();
        b.establish_children(&d, std::slice::from_ref(&label))
            .unwrap();
        let f = finish(b, &[target, label]);
        let c = context(&f, &[]);
        assert!(labels(&report(&f, &c)).is_empty());
    }
}
#[test]
fn exact_utf16_matching_preserves_surrogates_and_case() {
    for equal in [false, true] {
        let (mut b, d) = builder();
        let raw = DocumentString::from_utf16(vec![0xd800, 0, 65]);
        let other = if equal {
            raw.clone()
        } else {
            DocumentString::from_utf16(vec![0xd800, 0, 97])
        };
        let target = expanded(
            &mut b,
            &d,
            Namespace::html(),
            "input",
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), None, "id"),
                raw.clone(),
            )])
            .unwrap(),
        );
        let label = expanded(
            &mut b,
            &d,
            Namespace::html(),
            "label",
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(
                    Namespace::none(),
                    Some(DocumentString::from("retained")),
                    "for",
                ),
                other,
            )])
            .unwrap(),
        );
        for h in [&target, &label] {
            b.establish_children(h, &[]).unwrap();
        }
        b.establish_children(&d, &[label.clone(), target.clone()])
            .unwrap();
        let f = finish(b, &[target, label]);
        let c = context(&f, &[]);
        let r = report(&f, &c);
        if equal {
            assert_eq!(labels(&r), std::slice::from_ref(&f.ids[1]));
        } else {
            assert!(labels(&r).is_empty());
        }
        let HtmlCapturedNodeKind::Element(e) = f.capture.node(&f.ids[0]).unwrap().kind() else {
            panic!()
        };
        assert_eq!(e.attributes().as_slice()[0].value(), &raw);
    }
}
#[test]
fn namespaced_or_uppercase_label_and_for_names_do_not_match_by_local_coincidence() {
    let (mut b, d) = builder();
    let target = element(&mut b, &d, "input", &[("id", "f")]);
    let upper = element(&mut b, &d, "LABEL", &[("for", "f")]);
    let foreign = expanded(
        &mut b,
        &d,
        Namespace::svg(),
        "label",
        attrs(&[("for", "f")]),
    );
    let attr = expanded(
        &mut b,
        &d,
        Namespace::html(),
        "label",
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(
                Namespace::from_uri("http://www.w3.org/XML/1998/namespace"),
                None,
                "for",
            ),
            "f",
        )])
        .unwrap(),
    );
    for h in [&target, &upper, &foreign, &attr] {
        b.establish_children(h, &[]).unwrap();
    }
    b.establish_children(
        &d,
        &[upper.clone(), foreign.clone(), attr.clone(), target.clone()],
    )
    .unwrap();
    let f = finish(b, &[target, upper, foreign, attr]);
    let c = context(&f, &[(1, false)]);
    assert!(labels(&report(&f, &c)).is_empty());
    for (i, expected) in [
        (1, HtmlSemanticValue::Inapplicable),
        (
            2,
            HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace),
        ),
        (3, HtmlSemanticValue::Inapplicable),
    ] {
        let r = session(&f, &c, limits(), 0, HtmlSemanticCancellation::new())
            .query(&[HtmlSemanticRequest::new(
                f.ids[i].clone(),
                HtmlSemanticQuery::ControlLabels,
            )])
            .unwrap();
        assert_eq!(value(&r), &expected);
    }
}
#[test]
fn exact_caps_cover_retained_capacity_nested_scratch_and_atomic_failures() {
    let f = flat(
        &[
            ("input", &[("id", "abcdef")]),
            ("label", &[("for", "abcdef")]),
            ("label", &[("for", "abcdef")]),
        ],
        &[1, 2, 0],
    );
    let c = context(&f, &[]);
    let req = HtmlSemanticRequest::new(f.ids[0].clone(), HtmlSemanticQuery::ControlLabels);
    let baseline = report(&f, &c).usage();
    assert!(baseline.nested_output_records() >= 2);
    assert!(baseline.peak_scratch_records() > 0);
    assert_eq!(baseline.scratch_records(), 0);
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
        for cap in [exact, exact - 1] {
            let l = HtmlSemanticLimits::new(
                if resource == HtmlSemanticResource::Work {
                    cap
                } else {
                    10000000
                },
                if resource == HtmlSemanticResource::OutputRecords {
                    cap
                } else {
                    100
                },
                if resource == HtmlSemanticResource::ScratchRecords {
                    cap
                } else {
                    10000
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
                    10000
                },
            );
            let mut s = session(&f, &c, l, 6, HtmlSemanticCancellation::new());
            let r = s.query(std::slice::from_ref(&req));
            if cap == exact {
                assert_eq!(labels(&r.unwrap()), &[f.ids[1].clone(), f.ids[2].clone()]);
            } else {
                let e = r.unwrap_err();
                assert!(
                    matches!(e.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..}if *r==resource),
                    "{e:?}"
                );
                assert_eq!(e.qualification().unwrap().limits(), l);
                assert_eq!(s.query(&[]).unwrap_err(), e);
            }
            assert_eq!(s.usage().scratch_records(), 0);
        }
    }
    let mut s = session(&f, &c, limits(), 5, HtmlSemanticCancellation::new());
    let e = s
        .query(&[
            HtmlSemanticRequest::new(f.ids[0].clone(), HtmlSemanticQuery::Hidden),
            req,
        ])
        .unwrap_err();
    assert!(matches!(e.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(s.query(&[]).unwrap_err(), e);
    assert_eq!(s.usage().scratch_records(), 0);
}
#[test]
fn original_hidden_type_raw_is_admitted_before_null_short_circuit() {
    let (mut b, d) = builder();
    let target = element(&mut b, &d, "input", &[("type", "hidden")]);
    let f = finish(b, &[target]);
    let c = context(&f, &[]);
    for cap in [6, 5] {
        let mut s = session(&f, &c, limits(), cap, HtmlSemanticCancellation::new());
        let r = s.query(&[HtmlSemanticRequest::new(
            f.ids[0].clone(),
            HtmlSemanticQuery::ControlLabels,
        )]);
        if cap == 6 {
            assert_eq!(value(&r.unwrap()), &HtmlSemanticValue::Known(None));
        } else {
            assert!(matches!(
                r.unwrap_err().kind(),
                HtmlSemanticFailureKind::Attribute(_)
            ));
        }
    }
}
#[test]
fn cancellation_foreign_capture_and_duplicate_requests_keep_qualified_complete_outcomes() {
    let f = flat(&[("input", &[])], &[0]);
    let c = context(&f, &[]);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut s = session(&f, &c, limits(), 10, cancel);
    let req = HtmlSemanticRequest::new(f.ids[0].clone(), HtmlSemanticQuery::ControlLabels);
    let e = s.query(std::slice::from_ref(&req)).unwrap_err();
    assert_eq!(e.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(e.qualification().unwrap().capture(), f.capture.identity());
    assert_eq!(e.qualification().unwrap().context(), c.identity());
    assert_eq!(s.query(&[]).unwrap_err(), e);
    let other = flat(&[("input", &[])], &[0]);
    let mut s = session(&f, &c, limits(), 0, HtmlSemanticCancellation::new());
    let e = s
        .query(&[
            req.clone(),
            HtmlSemanticRequest::new(other.ids[0].clone(), HtmlSemanticQuery::ControlLabels),
        ])
        .unwrap_err();
    assert!(matches!(e.kind(), HtmlSemanticFailureKind::Capture(_)));
    assert_eq!(s.query(&[]).unwrap_err(), e);
    assert_eq!(s.usage().scratch_records(), 0);
    let mut s = session(&f, &c, limits(), 0, HtmlSemanticCancellation::new());
    let r = s.query(&[req.clone(), req]).unwrap();
    assert_eq!(r.results().len(), 2);
    assert_eq!(r.results()[0], r.results()[1]);
    assert!(matches!(
        HtmlSemanticSession::new(
            &other.capture,
            &c,
            HtmlValueInterpreter::new(0),
            limits(),
            HtmlSemanticCancellation::new()
        )
        .err()
        .unwrap()
        .kind(),
        HtmlSemanticFailureKind::ContextCaptureMismatch
    ));
}

#[test]
fn late_unavailable_subtree_returns_missing_instead_of_a_retained_label_prefix() {
    let (mut b, d) = builder();
    let target = element(&mut b, &d, "input", &[("id", "f")]);
    let label = element(&mut b, &d, "label", &[("for", "f")]);
    let branch = element(&mut b, &d, "div", &[]);
    for h in [&target, &label] {
        b.establish_children(h, &[]).unwrap();
    }
    b.establish_children(&d, &[label.clone(), target.clone(), branch.clone()])
        .unwrap();
    let f = finish(b, &[target, label, branch]);
    let c = context(&f, &[]);
    let mut s = session(&f, &c, limits(), 10, HtmlSemanticCancellation::new());
    let r = s
        .query(&[HtmlSemanticRequest::new(
            f.ids[0].clone(),
            HtmlSemanticQuery::ControlLabels,
        )])
        .unwrap();
    assert_eq!(
        value(&r),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(f.ids[2].clone()))
    );
    assert_eq!(s.usage().scratch_records(), 0);
    assert!(r.usage().nested_output_records() > 0);
}
#[test]
fn absent_target_id_skips_for_value_but_consulted_mismatching_for_keeps_raw_admission() {
    let huge = "x".repeat(1000);
    for has_id in [false, true] {
        let target_attrs = if has_id { &[("id", "f")][..] } else { &[][..] };
        let f = flat(
            &[("input", target_attrs), ("label", &[("for", &huge)])],
            &[1, 0],
        );
        let c = context(&f, &[]);
        let mut s = session(&f, &c, limits(), 0, HtmlSemanticCancellation::new());
        let r = s.query(&[HtmlSemanticRequest::new(
            f.ids[0].clone(),
            HtmlSemanticQuery::ControlLabels,
        )]);
        if has_id {
            assert!(matches!(
                r.unwrap_err().kind(),
                HtmlSemanticFailureKind::Attribute(_)
            ));
        } else {
            assert!(labels(&r.unwrap()).is_empty());
        }
        assert_eq!(s.usage().scratch_records(), 0);
    }
}
#[test]
fn button_labelability_does_not_consult_type_or_current_form_facts() {
    let huge = "x".repeat(1000);
    let f = flat(&[("button", &[("type", &huge)]), ("label", &[])], &[0, 1]);
    let c = context(&f, &[]);
    let mut s = session(&f, &c, limits(), 0, HtmlSemanticCancellation::new());
    let r = s
        .query(&[HtmlSemanticRequest::new(
            f.ids[0].clone(),
            HtmlSemanticQuery::ControlLabels,
        )])
        .unwrap();
    assert!(labels(&r).is_empty());
}
