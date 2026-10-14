#[path = "common/session.rs"]
mod shared_session;
use shared_session::session;
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
fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    local: &str,
    attrs: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    expanded(
        b,
        d,
        Namespace::html(),
        local,
        OrderedAttributes::new(attrs.iter().map(|(name, value)| {
            Attribute::new(QualifiedName::new(Namespace::none(), None, *name), *value)
        }))
        .unwrap(),
    )
}
fn expanded(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    namespace: Namespace,
    local: &str,
    attrs: OrderedAttributes,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(namespace, None, local),
        attrs,
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn fixture(local: &str, attrs: &[(&str, &str)], root: bool) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, local, attrs);
    if root {
        b.establish_root(&e).unwrap();
    }
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    (c, id)
}
fn facts(
    c: &HtmlCapture,
    values: &[(HtmlCaptureSubjectId, Option<DocumentString>)],
    units: usize,
) -> Result<HtmlLanguageDirectionFacts, HtmlLanguageDirectionError> {
    HtmlLanguageDirectionFacts::new(
        c,
        c.host_facts_revision().clone(),
        vec![],
        values
            .iter()
            .map(|(id, value)| HtmlControlCurrentValue::new(id.clone(), value.clone()))
            .collect(),
        vec![],
        HtmlLanguageDirectionLimits::new(100, 0, units, 1000000),
    )
}
fn context(
    c: &HtmlCapture,
    values: &[(HtmlCaptureSubjectId, Option<DocumentString>)],
) -> HtmlSemanticContext {
    let input = HtmlSemanticContextInput::new(vec![])
        .with_language_direction(facts(c, values, 10000).unwrap());
    HtmlSemanticContext::new(c, c.host_facts_revision().clone(), input, 0).unwrap()
}
fn limits(work: usize) -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(work, 100, 0, 1000, 1000000, 0)
}

fn request(id: &HtmlCaptureSubjectId, q: HtmlSemanticQuery) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), q)
}
fn report(
    c: &HtmlCapture,
    cx: &HtmlSemanticContext,
    id: &HtmlCaptureSubjectId,
    q: HtmlSemanticQuery,
    raw: usize,
) -> HtmlSemanticReport {
    session(c, cx, limits(1000000), raw, HtmlSemanticCancellation::new())
        .query(&[request(id, q)])
        .unwrap()
}
fn value(r: &HtmlSemanticReport) -> &HtmlSemanticValue<bool> {
    match r.results()[0].fact() {
        HtmlSemanticFact::InputTextValueMissing(v) | HtmlSemanticFact::TextareaValueMissing(v) => v,
        other => panic!("{other:?}"),
    }
}
fn query_for(local: &str) -> HtmlSemanticQuery {
    match local {
        "input" => HtmlSemanticQuery::InputTextValueMissing,
        "textarea" => HtmlSemanticQuery::TextareaValueMissing,
        _ => panic!("unsupported fixture role"),
    }
}

#[test]
fn all_six_input_states_and_textarea_require_a_mutable_empty_current_value() {
    for state in ["text", "search", "tel", "url", "email", "password"] {
        let (c, id) = fixture("input", &[("type", state), ("required", "false")], true);
        for (current, expected) in [("", true), ("x", false)] {
            let cx = context(&c, &[(id.clone(), Some(current.into()))]);
            assert_eq!(
                value(&report(&c, &cx, &id, query_for("input"), 100)),
                &HtmlSemanticValue::Known(expected)
            );
        }
    }
    let (c, id) = fixture("textarea", &[("required", "false")], true);
    let cx = context(&c, &[(id.clone(), Some("".into()))]);
    assert_eq!(
        value(&report(&c, &cx, &id, query_for("textarea"), 100)),
        &HtmlSemanticValue::Known(true)
    );
}

#[test]
fn absent_invalid_and_ascii_case_type_values_follow_the_existing_type_owner() {
    for state in [
        None,
        Some(""),
        Some("nonsense"),
        Some(" EMAIL "),
        Some("EmAiL"),
    ] {
        let mut attrs = vec![("required", "")];
        if let Some(state) = state {
            attrs.push(("type", state));
        }
        let (c, id) = fixture("input", &attrs, true);
        let cx = context(&c, &[(id.clone(), Some("".into()))]);
        assert_eq!(
            value(&report(&c, &cx, &id, query_for("input"), 100)),
            &HtmlSemanticValue::Known(true)
        );
    }
}

#[test]
fn required_and_readonly_are_presence_proofs_without_current_or_parent_facts() {
    for local in ["input", "textarea"] {
        for attrs in [
            vec![],
            vec![
                ("required", "FALSE"),
                ("readonly", "malformed and very long"),
            ],
        ] {
            let (c, id) = fixture(local, &attrs, false);
            let cx = context(&c, &[]);
            assert_eq!(
                value(&report(&c, &cx, &id, query_for(local), 0)),
                &HtmlSemanticValue::Known(false)
            );
        }
    }
}

#[test]
fn own_disabled_discharges_unavailable_current_value_and_ancestry() {
    for local in ["input", "textarea"] {
        let (c, id) = fixture(local, &[("required", ""), ("disabled", "false")], false);
        let cx = context(&c, &[(id.clone(), None)]);
        assert_eq!(
            value(&report(&c, &cx, &id, query_for(local), 100)),
            &HtmlSemanticValue::Known(false)
        );
    }
}

#[test]
fn any_exact_nonempty_units_prove_false_despite_unavailable_ancestry() {
    let values = [
        vec![32],
        vec![0],
        vec![13, 10],
        vec![0xd83d, 0xde00],
        vec![0xd800],
        vec![0xdc00],
    ];
    for local in ["input", "textarea"] {
        let (c, id) = fixture(local, &[("required", "")], false);
        for units in &values {
            let current = DocumentString::from_utf16(units.clone());
            let cx = context(&c, &[(id.clone(), Some(current))]);
            assert_eq!(
                value(&report(&c, &cx, &id, query_for(local), 0)),
                &HtmlSemanticValue::Known(false)
            );
        }
    }
}

#[test]
fn missing_current_empty_current_and_missing_parent_remain_distinct() {
    for local in ["input", "textarea"] {
        let (c, id) = fixture(local, &[("required", "")], false);
        for values in [vec![], vec![(id.clone(), None)]] {
            let cx = context(&c, &values);
            assert_eq!(
                value(&report(&c, &cx, &id, query_for(local), 100)),
                &HtmlSemanticValue::Missing(HtmlSemanticMissing::CurrentValue(id.clone()))
            );
        }
        let cx = context(&c, &[(id.clone(), Some("".into()))]);
        assert_eq!(
            value(&report(&c, &cx, &id, query_for(local), 100)),
            &HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(id))
        );
    }
}

#[test]
fn absent_supplement_does_not_mean_empty_and_known_mutability_keeps_current_missing() {
    let (c, id) = fixture("textarea", &[("required", "")], true);
    let cx = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]),
        0,
    )
    .unwrap();
    assert_eq!(
        value(&report(&c, &cx, &id, query_for("textarea"), 100)),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::CurrentValue(id))
    );
}

#[test]
fn content_value_and_descendant_default_text_cannot_replace_current_state() {
    for local in ["input", "textarea"] {
        for (authored, current, expected) in [("author", "", true), ("", "current", false)] {
            let (mut b, d) = builder();
            let e = element(&mut b, &d, local, &[("required", ""), ("value", authored)]);
            let text = b.add_text(&d, DocumentString::from(authored)).unwrap();
            b.establish_children(&e, &[text]).unwrap();
            b.establish_root(&e).unwrap();
            let c = b.finish().unwrap();
            let id = c.subject_id(&e).unwrap();
            let cx = context(&c, &[(id.clone(), Some(current.into()))]);
            assert_eq!(
                value(&report(&c, &cx, &id, query_for(local), 100)),
                &HtmlSemanticValue::Known(expected)
            );
        }
    }
}

#[test]
fn foreign_namespaces_wrong_roles_and_non_text_states_are_not_claimed_missingness() {
    let (mut b, d) = builder();
    let foreign = expanded(
        &mut b,
        &d,
        Namespace::svg(),
        "input",
        OrderedAttributes::new([]).unwrap(),
    );
    let div = element(&mut b, &d, "div", &[("required", "")]);
    let upper = element(&mut b, &d, "INPUT", &[("required", "")]);
    let text = b.add_text(&d, "".into()).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    for q in [query_for("input"), query_for("textarea")] {
        assert_eq!(
            value(&report(&c, &cx, &c.subject_id(&foreign).unwrap(), q, 100)),
            &HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
        );
        for e in [&div, &upper, &text, &d] {
            assert_eq!(
                value(&report(&c, &cx, &c.subject_id(e).unwrap(), q, 100)),
                &HtmlSemanticValue::Inapplicable
            );
        }
    }
    for state in [
        "hidden",
        "checkbox",
        "radio",
        "date",
        "month",
        "week",
        "time",
        "datetime-local",
        "number",
        "range",
        "color",
        "file",
        "submit",
        "reset",
        "button",
        "image",
    ] {
        let (c, id) = fixture("input", &[("type", state)], false);
        let cx = context(&c, &[]);
        assert_eq!(
            value(&report(&c, &cx, &id, query_for("input"), 100)),
            &HtmlSemanticValue::Inapplicable
        );
    }
    for (local, q) in [
        ("input", query_for("textarea")),
        ("textarea", query_for("input")),
    ] {
        let (c, id) = fixture(local, &[("required", "")], true);
        let cx = context(&c, &[]);
        assert_eq!(
            value(&report(&c, &cx, &id, q, 100)),
            &HtmlSemanticValue::Inapplicable
        );
    }
}

#[test]
fn expanded_boolean_names_are_exact_and_prefix_is_not_the_identity() {
    for (ns, prefix, local, expected) in [
        (Namespace::none(), Some("odd".into()), "required", true),
        (Namespace::none(), None, "REQUIRED", false),
        (Namespace::from_uri("urn:test"), None, "required", false),
    ] {
        let (mut b, d) = builder();
        let attrs = OrderedAttributes::new([
            Attribute::new(QualifiedName::new(ns, prefix, local), "false"),
            Attribute::new(
                QualifiedName::new(Namespace::from_uri("urn:test"), None, "readonly"),
                "",
            ),
        ])
        .unwrap();
        let e = expanded(&mut b, &d, Namespace::html(), "textarea", attrs);
        b.establish_root(&e).unwrap();
        let c = b.finish().unwrap();
        let id = c.subject_id(&e).unwrap();
        let cx = context(&c, &[(id.clone(), Some("".into()))]);
        assert_eq!(
            value(&report(&c, &cx, &id, query_for("textarea"), 100)),
            &HtmlSemanticValue::Known(expected)
        );
    }
}

#[test]
fn first_legend_exempts_only_its_branch_and_later_or_outer_fieldsets_can_disable() {
    let (mut b, d) = builder();
    let fs = element(&mut b, &d, "fieldset", &[("disabled", "")]);
    let l1 = element(&mut b, &d, "legend", &[]);
    let l2 = element(&mut b, &d, "legend", &[]);
    let a = element(&mut b, &d, "input", &[("required", "")]);
    let z = element(&mut b, &d, "textarea", &[("required", "")]);
    let outside = element(&mut b, &d, "input", &[("required", "")]);
    b.establish_children(&l1, std::slice::from_ref(&a)).unwrap();
    b.establish_children(&l2, std::slice::from_ref(&z)).unwrap();
    b.establish_children(&fs, &[l1, l2, outside.clone()])
        .unwrap();
    b.establish_root(&fs).unwrap();
    let c = b.finish().unwrap();
    let ids = [&a, &z, &outside].map(|e| c.subject_id(e).unwrap());
    let cx = context(
        &c,
        &ids.iter()
            .map(|id| (id.clone(), Some("".into())))
            .collect::<Vec<_>>(),
    );
    for (id, q, expected) in [
        (&ids[0], query_for("input"), true),
        (&ids[1], query_for("textarea"), false),
        (&ids[2], query_for("input"), false),
    ] {
        assert_eq!(
            value(&report(&c, &cx, id, q, 100)),
            &HtmlSemanticValue::Known(expected)
        );
    }
    let (mut b, d) = builder();
    let outer = element(&mut b, &d, "fieldset", &[("disabled", "")]);
    let inner = element(&mut b, &d, "fieldset", &[("disabled", "")]);
    let legend = element(&mut b, &d, "legend", &[]);
    let e = element(&mut b, &d, "textarea", &[("required", "")]);
    b.establish_children(&legend, std::slice::from_ref(&e))
        .unwrap();
    b.establish_children(&inner, &[legend]).unwrap();
    b.establish_children(&outer, &[inner]).unwrap();
    b.establish_root(&outer).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let cx = context(&c, &[(id.clone(), Some("".into()))]);
    assert_eq!(
        value(&report(&c, &cx, &id, query_for("textarea"), 100)),
        &HtmlSemanticValue::Known(false)
    );
}

#[test]
fn unknown_first_legend_order_is_influencing_only_for_an_empty_legend_branch() {
    let (mut b, d) = builder();
    let fs = element(&mut b, &d, "fieldset", &[("disabled", "")]);
    let legend = element(&mut b, &d, "legend", &[]);
    let e = element(&mut b, &d, "textarea", &[("required", "")]);
    b.establish_parent(&e, &legend).unwrap();
    b.establish_parent(&legend, &fs).unwrap();
    b.establish_root(&fs).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let fs_id = c.subject_id(&fs).unwrap();
    for (current, expected) in [
        (
            "",
            HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(fs_id)),
        ),
        ("x", HtmlSemanticValue::Known(false)),
    ] {
        let cx = context(&c, &[(id.clone(), Some(current.into()))]);
        assert_eq!(
            value(&report(&c, &cx, &id, query_for("textarea"), 100)),
            &expected
        );
    }
}

#[test]
fn datalist_candidacy_does_not_replace_mutability_or_empty_value() {
    let (mut b, d) = builder();
    let list = element(&mut b, &d, "datalist", &[]);
    let e = element(&mut b, &d, "input", &[("required", "")]);
    b.establish_children(&list, std::slice::from_ref(&e))
        .unwrap();
    b.establish_root(&list).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let cx = context(&c, &[(id.clone(), Some("".into()))]);
    let r = report(
        &c,
        &cx,
        &id,
        HtmlSemanticQuery::ConstraintValidationCandidate,
        100,
    );
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::ConstraintValidationCandidate(HtmlSemanticValue::Known(false))
    );
    assert_eq!(
        value(&report(&c, &cx, &id, query_for("input"), 100)),
        &HtmlSemanticValue::Known(true)
    );
}

#[test]
fn current_string_aggregate_cap_is_distinct_from_interpreter_attribute_limit() {
    let (c, id) = fixture("textarea", &[("required", "")], false);
    let values = [(
        id.clone(),
        Some(DocumentString::from_utf16(vec![0xd800; 128])),
    )];
    assert!(matches!(
        facts(&c, &values, 127),
        Err(HtmlLanguageDirectionError::CodeUnits {
            limit: 127,
            required: 128
        })
    ));
    let f = facts(&c, &values, 128).unwrap();
    let cx = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_language_direction(f),
        0,
    )
    .unwrap();
    assert_eq!(
        value(&report(&c, &cx, &id, query_for("textarea"), 2)),
        &HtmlSemanticValue::Known(false)
    );
    assert_eq!(values[0].1.as_ref().unwrap().code_units(), &[0xd800; 128]);
}

#[test]
fn role_and_type_cap_precede_required_absence_and_terminals_are_not_false_proofs() {
    for required in [false, true] {
        let attrs = if required {
            vec![("type", "password"), ("required", ""), ("readonly", "")]
        } else {
            vec![("type", "password")]
        };
        let (c, id) = fixture("input", &attrs, false);
        let cx = context(&c, &[]);
        let mut s = session(&c, &cx, limits(1000000), 7, HtmlSemanticCancellation::new());
        let failure = s.query(&[request(&id, query_for("input"))]).unwrap_err();
        assert!(matches!(
            failure.kind(),
            HtmlSemanticFailureKind::Attribute(_)
        ));
        assert_eq!(s.query(&[]).unwrap_err(), failure);
    }
    let (c, id) = fixture("textarea", &[("required", ""), ("disabled", "false")], true);
    let cx = context(&c, &[]);
    let mut s = session(&c, &cx, limits(1000000), 4, HtmlSemanticCancellation::new());
    let failure = s.query(&[request(&id, query_for("textarea"))]).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    assert_eq!(s.query(&[]).unwrap_err(), failure);
}

#[test]
fn literal_work_budgets_admit_complete_results_and_one_short_stops_before_write() {
    // E=113, A=144, namespace=28. Input:32+706+1+2265+1+1=3006.
    // Textarea absent required:32+1045+1+1=1079.
    // Required/empty textarea, unknown parent:4382 (see the independent scan costs below).
    for (local, attrs, current, work, expected) in [
        ("input", vec![], None, 3006, Some(false)),
        ("textarea", vec![], None, 1079, Some(false)),
        ("textarea", vec![("required", "")], Some(""), 4382, None),
    ] {
        let (c, id) = fixture(local, &attrs, false);
        let values = current
            .map(|v| vec![(id.clone(), Some(v.into()))])
            .unwrap_or_default();
        let cx = context(&c, &values);
        let expected = expected.map_or_else(
            || HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(id.clone())),
            HtmlSemanticValue::Known,
        );
        let req = request(&id, query_for(local));
        let mut exact = session(&c, &cx, limits(work), 100, HtmlSemanticCancellation::new());
        let r = exact.query(std::slice::from_ref(&req)).unwrap();
        assert_eq!(value(&r), &expected);
        assert_eq!(r.usage().work(), work);
        assert_eq!(r.usage().peak_scratch_records(), 0);
        let mut short = session(
            &c,
            &cx,
            limits(work - 1),
            100,
            HtmlSemanticCancellation::new(),
        );
        let failure = short.query(&[req]).unwrap_err();
        assert_eq!(
            failure.kind(),
            &HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::Work,
                limit: work - 1,
                required: work
            }
        );
        assert_eq!(short.query(&[]).unwrap_err(), failure);
        assert_eq!(short.usage().scratch_records(), 0);
    }
}

#[test]
fn ordered_current_records_cost_one_each_and_stop_before_an_unadmitted_visit() {
    let (mut b, d) = builder();
    let a = element(&mut b, &d, "textarea", &[("required", "")]);
    let z = element(&mut b, &d, "textarea", &[]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    let z = c.subject_id(&z).unwrap();
    for (values, work) in [
        (
            vec![(id.clone(), Some("x".into())), (z.clone(), Some("".into()))],
            1095,
        ),
        (
            vec![(z.clone(), Some("".into())), (id.clone(), Some("x".into()))],
            1096,
        ),
    ] {
        let cx = context(&c, &values);
        let r = report(&c, &cx, &id, query_for("textarea"), 100);
        assert_eq!(r.usage().work(), work);
        assert_eq!(value(&r), &HtmlSemanticValue::Known(false));
    }
    let cx = context(&c, &[(z, Some("".into())), (id.clone(), Some("x".into()))]);
    // Before lookup: reserve1+node1+namespace29+known1045+role1+
    // required9+presence1+readonly4+presence1=1092. One record is admitted.
    let mut s = session(&c, &cx, limits(1093), 100, HtmlSemanticCancellation::new());
    let failure = s.query(&[request(&id, query_for("textarea"))]).unwrap_err();
    assert_eq!(
        failure.kind(),
        &HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            limit: 1093,
            required: 1094
        }
    );
    assert_eq!(failure.usage().work(), 1093);
    assert_eq!(s.query(&[]).unwrap_err(), failure);
}

#[test]
fn output_admission_late_batch_failure_and_cancellation_remain_atomic_and_sticky() {
    let (mut b, d) = builder();
    let a = element(&mut b, &d, "textarea", &[]);
    let z = element(&mut b, &d, "input", &[("type", "password")]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    let z = c.subject_id(&z).unwrap();
    let cx = context(&c, &[]);
    let req = request(&id, query_for("textarea"));
    for (l, resource) in [
        (
            HtmlSemanticLimits::new(1000000, 0, 0, 1000, 1000000, 0),
            HtmlSemanticResource::OutputRecords,
        ),
        (
            HtmlSemanticLimits::new(1000000, 100, 0, 1000, 0, 0),
            HtmlSemanticResource::AllocatedBytes,
        ),
    ] {
        let mut s = session(&c, &cx, l, 100, HtmlSemanticCancellation::new());
        let failure = s.query(std::slice::from_ref(&req)).unwrap_err();
        assert!(
            matches!(failure.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r==resource)
        );
        assert_eq!(failure.usage().work(), 0);
        assert_eq!(s.query(&[]).unwrap_err(), failure);
    }
    let mut s = session(&c, &cx, limits(1000000), 7, HtmlSemanticCancellation::new());
    let earlier = s.query(std::slice::from_ref(&req)).unwrap();
    let failure = s
        .query(&[req.clone(), request(&z, query_for("input"))])
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    assert_eq!(value(&earlier), &HtmlSemanticValue::Known(false));
    assert_eq!(s.query(&[]).unwrap_err(), failure);
    let cancel = HtmlSemanticCancellation::new();
    let mut s = session(&c, &cx, limits(1000000), 100, cancel.clone());
    let earlier = s.query(std::slice::from_ref(&req)).unwrap();
    cancel.cancel();
    let failure = s.query(&[req]).unwrap_err();
    assert_eq!(failure.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(s.query(&[]).unwrap_err(), failure);
    assert_eq!(value(&earlier), &HtmlSemanticValue::Known(false));
}

#[test]
fn qualification_joins_exact_current_facts_and_rejects_other_capture_or_host_revision() {
    let (c, id) = fixture("textarea", &[("required", "")], true);
    let (other, foreign) = fixture("textarea", &[("required", "")], true);
    assert!(matches!(
        facts(&c, &[(foreign, Some("".into()))], 100),
        Err(HtmlLanguageDirectionError::Capture(_))
    ));
    let stale = HtmlLanguageDirectionFacts::new(
        &c,
        HtmlHostFactsRevision::fresh(),
        vec![],
        vec![HtmlControlCurrentValue::new(id.clone(), Some("".into()))],
        vec![],
        HtmlLanguageDirectionLimits::new(1, 0, 100, 1000000),
    );
    assert!(matches!(
        stale,
        Err(HtmlLanguageDirectionError::HostRevisionMismatch)
    ));
    let old = context(&c, &[(id.clone(), Some("".into()))]);
    let new = context(&c, &[(id.clone(), Some("x".into()))]);
    let before = report(&c, &old, &id, query_for("textarea"), 100);
    let after = report(&c, &new, &id, query_for("textarea"), 100);
    assert_eq!(value(&before), &HtmlSemanticValue::Known(true));
    assert_eq!(value(&after), &HtmlSemanticValue::Known(false));
    assert_ne!(
        before.qualification().context(),
        after.qualification().context()
    );
    assert_eq!(before.qualification().capture(), c.identity());
    assert_eq!(
        before.qualification().host_revision(),
        c.host_facts_revision()
    );
    assert_eq!(before.qualification().attribute_input_limit(), 100);
    assert!(
        matches!(HtmlSemanticSession::new(&other,&old,HtmlValueInterpreter::new(100),limits(1000000),HtmlSemanticCancellation::new()),Err(e) if *e.kind()==HtmlSemanticFailureKind::ContextCaptureMismatch)
    );
}

#[test]
fn actual_output_capacity_can_be_reused_at_its_exact_reported_byte_and_record_limits() {
    let (c, id) = fixture("textarea", &[], false);
    let cx = context(&c, &[]);
    let r = report(&c, &cx, &id, query_for("textarea"), 100);
    let bytes = r.usage().allocated_bytes();
    let records = r.usage().output_records();
    let req = request(&id, query_for("textarea"));
    let mut s = session(
        &c,
        &cx,
        HtmlSemanticLimits::new(1000000, records, 0, 1000, bytes, 0),
        100,
        HtmlSemanticCancellation::new(),
    );
    assert_eq!(
        value(&s.query(std::slice::from_ref(&req)).unwrap()),
        &HtmlSemanticValue::Known(false)
    );
    for (l, resource) in [
        (
            HtmlSemanticLimits::new(1000000, records - 1, 0, 1000, bytes, 0),
            HtmlSemanticResource::OutputRecords,
        ),
        (
            HtmlSemanticLimits::new(1000000, records, 0, 1000, bytes - 1, 0),
            HtmlSemanticResource::AllocatedBytes,
        ),
    ] {
        let mut s = session(&c, &cx, l, 100, HtmlSemanticCancellation::new());
        let e = s.query(std::slice::from_ref(&req)).unwrap_err();
        assert!(
            matches!(e.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r==resource)
        );
        assert_eq!(s.query(&[]).unwrap_err(), e);
    }
}

#[test]
fn each_query_retains_its_own_source_clause() {
    assert_eq!(
        query_for("input").source_clause(),
        "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-required"
    );
    assert_eq!(
        query_for("textarea").source_clause(),
        "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-textarea-required"
    );
}
