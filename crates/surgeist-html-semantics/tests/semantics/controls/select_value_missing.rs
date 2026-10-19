use crate::semantics::support::builder;
use crate::semantics::support::session;
use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlControlFacts, HtmlControlFactsLimits,
    HtmlControlFlag, HtmlOptionControlState, HtmlSemanticCancellation, HtmlSemanticContext,
    HtmlSemanticContextInput, HtmlSemanticDependencies, HtmlSemanticFact, HtmlSemanticFailureKind,
    HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport,
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
fn context(
    c: &HtmlCapture,
    selected: &[(HtmlCaptureSubjectId, Option<bool>)],
) -> HtmlSemanticContext {
    let facts = HtmlControlFacts::new(
        c,
        c.host_facts_revision().clone(),
        vec![],
        vec![],
        selected
            .iter()
            .map(|(id, value)| HtmlOptionControlState::new(id.clone(), *value, None))
            .collect(),
        vec![],
        HtmlControlFactsLimits::new(1000, 1000000),
    )
    .unwrap();
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_controls(facts),
        0,
    )
    .unwrap()
}
fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20000000, 100, 1000, 0, 1000000, 100000)
}

fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::SelectValueMissing)
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
fn known(r: &HtmlSemanticReport) -> bool {
    match r.results()[0].fact() {
        HtmlSemanticFact::SelectValueMissing(HtmlSemanticValue::Known(v)) => *v,
        other => panic!("expected known missingness, got {other:?}"),
    }
}
fn missing(r: &HtmlSemanticReport) -> &HtmlSemanticMissing {
    match r.results()[0].fact() {
        HtmlSemanticFact::SelectValueMissing(HtmlSemanticValue::Missing(v)) => v,
        other => panic!("expected missing evidence, got {other:?}"),
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
fn absent_required_proves_false_without_unavailable_order_or_attribute_content() {
    let (mut b, d) = builder();
    let s = html(
        &mut b,
        &d,
        "select",
        &[("size", "enormous"), ("multiple", "invalid spelling")],
    );
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    let r = report(&c, &cx, &c.subject_id(&s).unwrap(), 0);
    assert!(!known(&r));
    assert_eq!(r.usage().peak_scratch_records(), 0);
    assert_eq!(r.usage().nested_output_records(), 0);
}

#[test]
fn required_empty_list_is_missing_without_current_state_or_size_admission() {
    let (mut b, d) = builder();
    let s = html(
        &mut b,
        &d,
        "select",
        &[("required", "nonboolean"), ("size", "too long")],
    );
    children(&mut b, &s, &[]);
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    let r = report(&c, &cx, &c.subject_id(&s).unwrap(), 0);
    assert!(known(&r));
    assert_eq!(r.usage().peak_scratch_records(), 0);
}

#[test]
fn all_false_current_selectedness_proves_missing_despite_authored_selected() {
    let (c, s, o) = fixture(
        &[("required", "false"), ("size", "unread")],
        &[("selected", ""), ("value", "huge")],
        None,
    );
    let cx = context(&c, &[(o, Some(false))]);
    assert!(known(&report(&c, &cx, &s, 0)));
}

#[test]
fn selectedness_unavailable_is_distinct_from_established_false() {
    let (c, s, o) = fixture(
        &[("required", "")],
        &[("selected", ""), ("value", "")],
        None,
    );
    for records in [vec![], vec![(o.clone(), None)]] {
        let cx = context(&c, &records);
        assert!(
            matches!(missing(&report(&c, &cx, &s, 0)), HtmlSemanticMissing::ControlState { subject, flag: HtmlControlFlag::Selectedness } if subject == &o)
        );
    }
}

#[test]
fn sole_selected_placeholder_and_nonplaceholder_have_independent_literal_results() {
    for (value, expected) in [("", true), (" ", false), ("x", false)] {
        let (c, s, o) = fixture(&[("required", "")], &[("value", value)], None);
        let cx = context(&c, &[(o, Some(true))]);
        assert_eq!(known(&report(&c, &cx, &s, value.len())), expected);
    }
}

#[test]
fn display_size_recovery_and_multiple_use_the_placeholder_definition_not_author_requirement() {
    for (size, multiple, expected) in [
        (None, false, true),
        (None, true, false),
        (Some("1"), true, true),
        (Some("+1tail"), true, true),
        (Some(" \t01"), false, true),
        (Some("0"), false, false),
        (Some("2"), false, false),
        (Some("99999999999999999999999999999999"), false, false),
        (Some("nope"), false, true),
        (Some("nope"), true, false),
        (Some("-1"), false, true),
        (Some("-0"), false, false),
    ] {
        let mut attrs = vec![("required", "")];
        if let Some(size) = size {
            attrs.push(("size", size));
        }
        if multiple {
            attrs.push(("multiple", "unread oversized spelling"));
        }
        let (c, s, o) = fixture(&attrs, &[("value", "")], None);
        let cx = context(&c, &[(o, Some(true))]);
        assert_eq!(
            known(&report(&c, &cx, &s, 100)),
            expected,
            "{size:?}/{multiple}"
        );
    }
}

#[test]
fn selected_first_option_inside_optgroup_is_not_a_placeholder() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let g = html(&mut b, &d, "optgroup", &[]);
    let o = html(&mut b, &d, "option", &[("value", "unread")]);
    children(&mut b, &g, std::slice::from_ref(&o));
    children(&mut b, &s, &[g]);
    let c = b.finish().unwrap();
    let cx = context(&c, &[(c.subject_id(&o).unwrap(), Some(true))]);
    assert!(!known(&report(&c, &cx, &c.subject_id(&s).unwrap(), 0)));
}

#[test]
fn selected_nonfirst_empty_option_proves_false_despite_unknown_first_state_and_later_order() {
    let (mut b, d) = builder();
    let s = html(
        &mut b,
        &d,
        "select",
        &[("required", ""), ("size", "unread")],
    );
    let a = html(&mut b, &d, "option", &[("value", "unread")]);
    let o = html(&mut b, &d, "option", &[("value", "")]);
    let later = html(&mut b, &d, "div", &[]);
    children(&mut b, &s, &[a, o.clone(), later]);
    let c = b.finish().unwrap();
    let cx = context(&c, &[(c.subject_id(&o).unwrap(), Some(true))]);
    assert!(!known(&report(&c, &cx, &c.subject_id(&s).unwrap(), 0)));
}

#[test]
fn selected_nonplaceholder_first_proves_false_despite_unknown_other_selectedness() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let a = html(&mut b, &d, "option", &[("value", "nonempty")]);
    let o = html(&mut b, &d, "option", &[]);
    children(&mut b, &s, &[a.clone(), o]);
    let c = b.finish().unwrap();
    let cx = context(&c, &[(c.subject_id(&a).unwrap(), Some(true))]);
    assert!(!known(&report(&c, &cx, &c.subject_id(&s).unwrap(), 8)));
}

#[test]
fn selected_placeholder_with_unknown_other_selectedness_remains_missing() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let a = html(&mut b, &d, "option", &[("value", "")]);
    let o = html(&mut b, &d, "option", &[]);
    children(&mut b, &s, &[a.clone(), o.clone()]);
    let c = b.finish().unwrap();
    let oid = c.subject_id(&o).unwrap();
    let cx = context(&c, &[(c.subject_id(&a).unwrap(), Some(true))]);
    assert!(
        matches!(missing(&report(&c, &cx, &c.subject_id(&s).unwrap(), 0)), HtmlSemanticMissing::ControlState { subject, .. } if subject == &oid)
    );
}

#[test]
fn two_selected_witnesses_prove_false_across_an_earlier_unknown_subtree() {
    let (mut b, d) = builder();
    let s = html(
        &mut b,
        &d,
        "select",
        &[("required", ""), ("size", "unread")],
    );
    let gap = html(&mut b, &d, "div", &[]);
    let a = html(&mut b, &d, "option", &[("value", "unread")]);
    let o = html(&mut b, &d, "option", &[]);
    let later = html(&mut b, &d, "div", &[]);
    children(&mut b, &s, &[gap, a.clone(), o.clone(), later]);
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        &[
            (c.subject_id(&a).unwrap(), Some(true)),
            (c.subject_id(&o).unwrap(), Some(true)),
        ],
    );
    assert!(!known(&report(&c, &cx, &c.subject_id(&s).unwrap(), 0)));
}

#[test]
fn selected_member_inside_optgroup_proves_false_despite_earlier_unavailable_subtree() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let gap = html(&mut b, &d, "div", &[]);
    let group = html(&mut b, &d, "optgroup", &[]);
    let o = html(&mut b, &d, "option", &[]);
    children(&mut b, &group, std::slice::from_ref(&o));
    children(&mut b, &s, &[gap, group]);
    let c = b.finish().unwrap();
    let sid = c.subject_id(&s).unwrap();
    let cx = context(&c, &[(c.subject_id(&o).unwrap(), Some(true))]);
    // No completion of gap can make this selected option a placeholder: its
    // actual optgroup ancestry is enough, independently of value or firstness.
    let r = report(&c, &cx, &sid, 0);
    assert!(!known(&r));
    assert_eq!(r.usage().nested_output_records(), 0);
    assert_eq!(r.usage().scratch_records(), 0);
    assert_eq!(r.capture(), c.identity());
    assert_eq!(r.context(), cx.identity());
}

#[test]
fn propagated_optgroup_witness_skips_size_value_and_later_unavailable_order() {
    let (mut b, d) = builder();
    let s = html(
        &mut b,
        &d,
        "select",
        &[("required", ""), ("size", "unread content")],
    );
    let gap = html(&mut b, &d, "div", &[]);
    let group = html(&mut b, &d, "optgroup", &[]);
    let wrapper = html(&mut b, &d, "div", &[]);
    let o = html(&mut b, &d, "option", &[("value", "unread content")]);
    let later = html(&mut b, &d, "div", &[]);
    children(&mut b, &wrapper, std::slice::from_ref(&o));
    children(&mut b, &group, &[wrapper]);
    children(&mut b, &s, &[gap, group, later]);
    let c = b.finish().unwrap();
    let cx = context(&c, &[(c.subject_id(&o).unwrap(), Some(true))]);
    let r = report(&c, &cx, &c.subject_id(&s).unwrap(), 0);
    assert!(!known(&r));
    assert_eq!(r.usage().nested_output_records(), 0);
    assert_eq!(r.usage().scratch_records(), 0);
}

#[test]
fn optgroup_ancestry_without_known_selection_does_not_prove_false() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let gap = html(&mut b, &d, "div", &[]);
    let group = html(&mut b, &d, "optgroup", &[]);
    let o = html(&mut b, &d, "option", &[]);
    children(&mut b, &group, std::slice::from_ref(&o));
    children(&mut b, &s, &[gap.clone(), group]);
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    assert_eq!(
        missing(&report(&c, &cx, &c.subject_id(&s).unwrap(), 0)),
        &HtmlSemanticMissing::OrderedChildren(c.subject_id(&gap).unwrap())
    );
}

#[test]
fn nonempty_selected_value_proves_false_despite_earlier_unavailable_subtree() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let gap = html(&mut b, &d, "div", &[]);
    let o = html(&mut b, &d, "option", &[("value", "x")]);
    children(&mut b, &s, &[gap, o.clone()]);
    let c = b.finish().unwrap();
    let cx = context(&c, &[(c.subject_id(&o).unwrap(), Some(true))]);
    // This selected member cannot be the placeholder regardless of firstness.
    let r = report(&c, &cx, &c.subject_id(&s).unwrap(), 1);
    assert!(!known(&r));
    assert_eq!(r.usage().scratch_records(), 0);
    assert_eq!(r.capture(), c.identity());
    assert_eq!(r.context(), cx.identity());
}

#[test]
fn encountered_predecessor_proves_selected_member_nonfirst_across_earlier_unknown_order() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let gap = html(&mut b, &d, "div", &[]);
    let a = html(&mut b, &d, "option", &[]);
    let selected = html(&mut b, &d, "option", &[("value", "")]);
    children(&mut b, &s, &[gap, a.clone(), selected.clone()]);
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        &[
            (c.subject_id(&a).unwrap(), Some(false)),
            (c.subject_id(&selected).unwrap(), Some(true)),
        ],
    );
    // A precedes the selected member whether or not gap contains other options.
    let r = report(&c, &cx, &c.subject_id(&s).unwrap(), 0);
    assert!(!known(&r));
    assert_eq!(r.usage().scratch_records(), 0);
    assert_eq!(r.usage().nested_output_records(), 0);
}

#[test]
fn an_earlier_unknown_subtree_cannot_be_skipped_when_establishing_first_option() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let gap = html(&mut b, &d, "div", &[]);
    let a = html(&mut b, &d, "option", &[("value", "")]);
    children(&mut b, &s, &[gap.clone(), a.clone()]);
    let c = b.finish().unwrap();
    let cx = context(&c, &[(c.subject_id(&a).unwrap(), Some(true))]);
    assert_eq!(
        missing(&report(&c, &cx, &c.subject_id(&s).unwrap(), 0)),
        &HtmlSemanticMissing::OrderedChildren(c.subject_id(&gap).unwrap())
    );
}

#[test]
fn unavailable_subtree_with_no_known_selection_is_not_an_empty_list() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let gap = html(&mut b, &d, "div", &[]);
    children(&mut b, &s, std::slice::from_ref(&gap));
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    assert_eq!(
        missing(&report(&c, &cx, &c.subject_id(&s).unwrap(), 0)),
        &HtmlSemanticMissing::OrderedChildren(c.subject_id(&gap).unwrap())
    );
}

#[test]
fn exact_pruning_ignores_selection_beneath_excluded_subtrees() {
    for excluded in ["select", "hr", "option", "datalist"] {
        let (mut b, d) = builder();
        let s = html(&mut b, &d, "select", &[("required", "")]);
        let branch = html(&mut b, &d, excluded, &[]);
        let hidden = html(&mut b, &d, "option", &[("value", "x")]);
        children(&mut b, &branch, std::slice::from_ref(&hidden));
        children(&mut b, &s, std::slice::from_ref(&branch));
        let c = b.finish().unwrap();
        let mut state = vec![(c.subject_id(&hidden).unwrap(), Some(true))];
        if excluded == "option" {
            state.push((c.subject_id(&branch).unwrap(), Some(false)));
        }
        let cx = context(&c, &state);
        assert!(
            known(&report(&c, &cx, &c.subject_id(&s).unwrap(), 0)),
            "{excluded}"
        );
    }
}

#[test]
fn nested_optgroups_are_pruned_but_options_under_foreign_wrappers_are_members() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let outer = html(&mut b, &d, "optgroup", &[]);
    let inner = html(&mut b, &d, "optgroup", &[]);
    let excluded = html(&mut b, &d, "option", &[]);
    children(&mut b, &inner, std::slice::from_ref(&excluded));
    children(&mut b, &outer, &[inner]);
    let foreign = element(
        &mut b,
        &d,
        Namespace::svg(),
        "g",
        OrderedAttributes::new([]).unwrap(),
    );
    let selected = html(&mut b, &d, "option", &[("value", "")]);
    children(&mut b, &foreign, std::slice::from_ref(&selected));
    children(&mut b, &s, &[outer, foreign]);
    let c = b.finish().unwrap();
    let cx = context(&c, &[(c.subject_id(&selected).unwrap(), Some(true))]);
    assert!(known(&report(&c, &cx, &c.subject_id(&s).unwrap(), 0)));
}

#[test]
fn disabledness_does_not_replace_the_source_missingness_predicate() {
    let (c, s, o) = fixture(
        &[("required", ""), ("disabled", "unread")],
        &[("value", ""), ("disabled", "unread")],
        None,
    );
    let cx = context(&c, &[(o, Some(true))]);
    assert!(known(&report(&c, &cx, &s, 0)));
}

#[test]
fn normalized_descendant_value_is_consumed_only_for_possible_placeholder() {
    for (text, expected) in [(" \t\n\r\u{c}", true), ("\u{a0}", false), ("a", false)] {
        let (c, s, o) = fixture(&[("required", "")], &[], Some(text));
        let cx = context(&c, &[(o, Some(true))]);
        let r = report(&c, &cx, &s, 0);
        assert_eq!(known(&r), expected);
        // All-ASCII-whitespace collapses to empty without an output buffer.
        if !expected {
            assert!(r.usage().nested_output_records() > 0);
        }
        assert_eq!(r.usage().scratch_records(), 0);
    }
}

#[test]
fn missing_option_text_is_influencing_only_when_a_selected_placeholder_is_possible() {
    let (c, s, o) = fixture(&[("required", "")], &[], None);
    let cx = context(&c, &[(o.clone(), Some(true))]);
    assert_eq!(
        missing(&report(&c, &cx, &s, 0)),
        &HtmlSemanticMissing::OrderedChildren(o.clone())
    );
    let cx = context(&c, &[(o, Some(false))]);
    assert!(known(&report(&c, &cx, &s, 0)));
}

#[test]
fn role_and_namespace_are_exact_and_foreign_subjects_are_rejected() {
    let (mut b, d) = builder();
    let div = html(&mut b, &d, "div", &[]);
    let upper = html(&mut b, &d, "SELECT", &[]);
    let foreign = element(
        &mut b,
        &d,
        Namespace::svg(),
        "select",
        OrderedAttributes::new([]).unwrap(),
    );
    let text = b.add_text(&d, "x".into()).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    for id in [&div, &upper, &text, &d] {
        assert_eq!(
            report(&c, &cx, &c.subject_id(id).unwrap(), 0).results()[0].fact(),
            &HtmlSemanticFact::SelectValueMissing(HtmlSemanticValue::Inapplicable)
        );
    }
    assert_eq!(
        report(&c, &cx, &c.subject_id(&foreign).unwrap(), 0).results()[0].fact(),
        &HtmlSemanticFact::SelectValueMissing(HtmlSemanticValue::Unsupported(
            HtmlSemanticUnsupported::ForeignNamespace
        ))
    );
    let (other, s, _) = fixture(&[], &[], None);
    let mut ss = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    assert!(matches!(
        ss.query(&[request(&s)]).unwrap_err().kind(),
        HtmlSemanticFailureKind::Capture(_)
    ));
    assert_ne!(other.identity(), c.identity());
}

#[test]
fn changed_current_state_at_one_capture_recomputes_with_distinct_context_identity() {
    let (c, s, o) = fixture(&[("required", "")], &[("value", "x")], None);
    let a = context(&c, &[(o.clone(), Some(false))]);
    let b = context(&c, &[(o, Some(true))]);
    let ra = report(&c, &a, &s, 1);
    let rb = report(&c, &b, &s, 1);
    assert!(known(&ra));
    assert!(!known(&rb));
    assert_eq!(ra.capture(), rb.capture());
    assert_ne!(ra.context(), rb.context());
    assert_eq!(
        ra.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
    assert_eq!(rb.host_revision(), c.host_facts_revision());
    assert_eq!(rb.results()[0].request().subject(), &s);
    assert_eq!(
        rb.results()[0].request().query().source_clause(),
        HtmlSemanticQuery::SelectValueMissing.source_clause()
    );
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
    )
}

#[test]
fn exact_and_one_short_work_scratch_output_nested_and_bytes_bound_the_real_text_pipeline() {
    let (c, s, o) = fixture(&[("required", "")], &[], Some("not empty normalized text"));
    let cx = context(&c, &[(o, Some(true))]);
    let baseline = report(&c, &cx, &s, 0);
    assert!(!known(&baseline));
    let u = baseline.usage();
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
        assert!(exact > 0);
        for cap in [exact, exact - 1] {
            let mut ss = session(
                &c,
                &cx,
                resource_limits(resource, cap),
                0,
                HtmlSemanticCancellation::new(),
            );
            let r = ss.query(&[request(&s)]);
            if cap == exact {
                assert!(!known(&r.unwrap()));
            } else {
                let f = r.unwrap_err();
                assert!(
                    matches!(f.kind(), HtmlSemanticFailureKind::LimitExceeded { resource: actual, .. } if actual == &resource)
                );
                assert_eq!(
                    f.qualification().unwrap().limits(),
                    resource_limits(resource, cap)
                );
                assert_eq!(ss.query(&[]).unwrap_err(), f);
            }
            assert_eq!(ss.usage().scratch_records(), 0);
        }
    }
}

#[test]
fn consulted_size_and_attribute_value_whole_raw_caps_remain_terminal() {
    for (select, option, cap) in [
        (
            vec![("required", ""), ("size", "1tail")],
            vec![("value", "")],
            4,
        ),
        (vec![("required", "")], vec![("value", "long")], 3),
    ] {
        let (c, s, o) = fixture(&select, &option, None);
        let cx = context(&c, &[(o, Some(true))]);
        assert_eq!(known(&report(&c, &cx, &s, 5)), option[0].1.is_empty());
        let mut ss = session(&c, &cx, limits(), cap, HtmlSemanticCancellation::new());
        let f = ss.query(&[request(&s)]).unwrap_err();
        assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
        assert_eq!(f.qualification().unwrap().attribute_input_limit(), cap);
        assert_eq!(ss.query(&[]).unwrap_err(), f);
        assert_eq!(ss.usage().scratch_records(), 0);
    }
}

#[test]
fn duplicate_queries_retain_discarded_intermediate_cost_and_fail_atomically_on_second() {
    let (c, s, o) = fixture(&[("required", "")], &[], Some("text"));
    let cx = context(&c, &[(o, Some(true))]);
    let single = report(&c, &cx, &s, 0).usage();
    let mut ss = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let r = ss.query(&[request(&s), request(&s)]).unwrap();
    assert_eq!(r.results().len(), 2);
    assert_eq!(
        r.usage().nested_output_records(),
        2 * single.nested_output_records()
    );
    assert!(r.usage().allocated_bytes() > single.allocated_bytes());
    let mut ss = session(
        &c,
        &cx,
        resource_limits(
            HtmlSemanticResource::NestedOutputRecords,
            single.nested_output_records(),
        ),
        0,
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
fn late_batch_raw_failure_and_cancellation_are_sticky_and_qualified() {
    let (mut b, d) = builder();
    let first = html(&mut b, &d, "select", &[]);
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let o = html(&mut b, &d, "option", &[("value", "long")]);
    children(&mut b, &s, std::slice::from_ref(&o));
    let c = b.finish().unwrap();
    let cx = context(&c, &[(c.subject_id(&o).unwrap(), Some(true))]);
    let first = c.subject_id(&first).unwrap();
    let s = c.subject_id(&s).unwrap();
    let mut ss = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let f = ss.query(&[request(&first), request(&s)]).unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(f.qualification().unwrap().capture(), c.identity());
    assert_eq!(f.qualification().unwrap().context(), cx.identity());
    assert_eq!(ss.query(&[]).unwrap_err(), f);
    assert_eq!(ss.usage().scratch_records(), 0);
    let cancellation = HtmlSemanticCancellation::new();
    let mut ss = session(&c, &cx, limits(), 100, cancellation.clone());
    assert!(!known(&ss.query(&[request(&first)]).unwrap()));
    cancellation.cancel();
    let f = ss.query(&[request(&s)]).unwrap_err();
    assert_eq!(f.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(ss.query(&[]).unwrap_err(), f);
    assert_eq!(ss.usage().scratch_records(), 0);
}

#[test]
fn unavailable_root_order_is_not_known_no_selection() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let c = b.finish().unwrap();
    let sid = c.subject_id(&s).unwrap();
    let cx = context(&c, &[]);
    assert_eq!(
        missing(&report(&c, &cx, &sid, 0)),
        &HtmlSemanticMissing::OrderedChildren(sid)
    );
}

#[test]
fn no_placeholder_display_size_proves_false_with_selected_member_and_unknown_earlier_order() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", ""), ("size", "2")]);
    let gap = html(&mut b, &d, "div", &[]);
    let o = html(&mut b, &d, "option", &[]);
    children(&mut b, &s, &[gap, o.clone()]);
    let c = b.finish().unwrap();
    let cx = context(&c, &[(c.subject_id(&o).unwrap(), Some(true))]);
    assert!(!known(&report(&c, &cx, &c.subject_id(&s).unwrap(), 1)));
}

#[test]
fn namespace_and_case_lookalike_required_attributes_do_not_establish_presence() {
    for (namespace, local) in [
        (Namespace::svg(), "required"),
        (Namespace::none(), "REQUIRED"),
    ] {
        let (mut b, d) = builder();
        let s = element(
            &mut b,
            &d,
            Namespace::html(),
            "select",
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(namespace, None, local),
                "huge unread spelling",
            )])
            .unwrap(),
        );
        let c = b.finish().unwrap();
        let cx = context(&c, &[]);
        assert!(!known(&report(&c, &cx, &c.subject_id(&s).unwrap(), 0)));
    }
}

#[test]
fn foreign_option_lookalike_has_no_selectedness_prerequisite() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let o = element(
        &mut b,
        &d,
        Namespace::svg(),
        "option",
        OrderedAttributes::new([]).unwrap(),
    );
    children(&mut b, &o, &[]);
    children(&mut b, &s, &[o]);
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    assert!(known(&report(&c, &cx, &c.subject_id(&s).unwrap(), 0)));
}

#[test]
fn isolated_surrogate_option_value_is_nonempty_and_admitted_as_exact_utf16() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let raw = DocumentString::from_utf16(vec![0xd800]);
    let o = element(
        &mut b,
        &d,
        Namespace::html(),
        "option",
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::none(), None, "value"),
            raw.clone(),
        )])
        .unwrap(),
    );
    children(&mut b, &s, std::slice::from_ref(&o));
    let c = b.finish().unwrap();
    let cx = context(&c, &[(c.subject_id(&o).unwrap(), Some(true))]);
    let sid = c.subject_id(&s).unwrap();
    assert!(!known(&report(&c, &cx, &sid, 1)));
    let mut ss = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    assert!(matches!(
        ss.query(&[request(&sid)]).unwrap_err().kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    assert_eq!(raw.code_units(), &[0xd800]);
}

#[test]
fn changed_text_capture_and_host_inputs_recompute_distinct_results_and_qualification() {
    let (a, s, o) = fixture(&[("required", "")], &[], Some(" \t"));
    let (b, t, p) = fixture(&[("required", "")], &[], Some("x"));
    let ax = context(&a, &[(o, Some(true))]);
    let bx = context(&b, &[(p, Some(true))]);
    let ra = report(&a, &ax, &s, 0);
    let rb = report(&b, &bx, &t, 0);
    assert!(known(&ra));
    assert!(!known(&rb));
    assert_ne!(ra.capture(), rb.capture());
    assert_ne!(ra.context(), rb.context());
    assert_ne!(ra.host_revision(), rb.host_revision());
    assert_eq!(ra.attribute_input_limit(), 0);
    assert_eq!(
        ra.source_revision(),
        "877d6146487f15b15bc8dd0e3c625063a37b456a"
    );
}

#[test]
fn nonterminal_missing_evidence_never_hides_a_later_terminal_work_stop() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let gap = html(&mut b, &d, "div", &[]);
    let o = html(&mut b, &d, "option", &[]);
    children(&mut b, &s, &[gap.clone(), o]);
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    let sid = c.subject_id(&s).unwrap();
    let r = report(&c, &cx, &sid, 0);
    assert_eq!(
        missing(&r),
        &HtmlSemanticMissing::OrderedChildren(c.subject_id(&gap).unwrap())
    );
    let mut ss = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::Work, r.usage().work() - 1),
        0,
        HtmlSemanticCancellation::new(),
    );
    let f = ss.query(&[request(&sid)]).unwrap_err();
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            ..
        }
    ));
    assert_eq!(ss.query(&[]).unwrap_err(), f);
    assert_eq!(ss.usage().scratch_records(), 0);
}

fn gap_fixture(
    option_attrs: &[(&str, &str)],
    text: Option<&str>,
) -> (
    HtmlCapture,
    HtmlCaptureSubjectId,
    HtmlCaptureSubjectId,
    HtmlCaptureSubjectId,
) {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let gap = html(&mut b, &d, "div", &[]);
    let o = html(&mut b, &d, "option", option_attrs);
    if let Some(text) = text {
        let t = b.add_text(&d, text.into()).unwrap();
        children(&mut b, &o, &[t]);
    }
    children(&mut b, &s, &[gap.clone(), o.clone()]);
    let c = b.finish().unwrap();
    let sid = c.subject_id(&s).unwrap();
    let oid = c.subject_id(&o).unwrap();
    let gid = c.subject_id(&gap).unwrap();
    (c, sid, oid, gid)
}

#[test]
fn normalized_selected_text_proves_false_across_unknown_firstness() {
    for text in [" \tx\n ", "\u{a0}"] {
        let (c, s, o, _) = gap_fixture(&[], Some(text));
        let cx = context(&c, &[(o, Some(true))]);
        let r = report(&c, &cx, &s, 0);
        assert!(!known(&r));
        assert!(r.usage().nested_output_records() > 0);
        assert_eq!(r.usage().scratch_records(), 0);
    }
}

#[test]
fn empty_and_unavailable_selected_values_cannot_overcome_unknown_firstness() {
    for (attrs, text) in [
        (vec![("value", "")], None),
        (vec![], Some(" \t\n\r\u{c}")),
        (vec![], None),
    ] {
        let (c, s, o, gap) = gap_fixture(&attrs, text);
        let cx = context(&c, &[(o, Some(true))]);
        let r = report(&c, &cx, &s, 0);
        assert_eq!(missing(&r), &HtmlSemanticMissing::OrderedChildren(gap));
        assert_eq!(r.usage().scratch_records(), 0);
    }
}

#[test]
fn absent_placeholder_display_proof_still_skips_oversized_selected_value() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", ""), ("size", "2")]);
    let gap = html(&mut b, &d, "div", &[]);
    let o = html(
        &mut b,
        &d,
        "option",
        &[("value", "oversized irrelevant value")],
    );
    children(&mut b, &s, &[gap, o.clone()]);
    let c = b.finish().unwrap();
    let cx = context(&c, &[(c.subject_id(&o).unwrap(), Some(true))]);
    assert!(!known(&report(&c, &cx, &c.subject_id(&s).unwrap(), 1)));
}

#[test]
fn fallback_selected_attribute_whole_raw_admission_remains_terminal_and_sticky() {
    let (c, s, o, _) = gap_fixture(&[("value", "long")], None);
    let cx = context(&c, &[(o, Some(true))]);
    assert!(!known(&report(&c, &cx, &s, 4)));
    let mut ss = session(&c, &cx, limits(), 3, HtmlSemanticCancellation::new());
    let f = ss.query(&[request(&s)]).unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(f.qualification().unwrap().attribute_input_limit(), 3);
    assert_eq!(ss.query(&[]).unwrap_err(), f);
    assert_eq!(ss.usage().scratch_records(), 0);
}

#[test]
fn fallback_descendant_pipeline_has_exact_and_one_short_physical_resource_boundaries() {
    let (c, s, o, _) = gap_fixture(&[], Some("nonempty normalized fallback text"));
    let cx = context(&c, &[(o, Some(true))]);
    let baseline = report(&c, &cx, &s, 0);
    assert!(!known(&baseline));
    let u = baseline.usage();
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
        assert!(exact > 0);
        for cap in [exact, exact - 1] {
            let mut ss = session(
                &c,
                &cx,
                resource_limits(resource, cap),
                0,
                HtmlSemanticCancellation::new(),
            );
            let result = ss.query(&[request(&s)]);
            if cap == exact {
                assert!(!known(&result.unwrap()));
            } else {
                let f = result.unwrap_err();
                assert!(
                    matches!(f.kind(), HtmlSemanticFailureKind::LimitExceeded { resource: actual, .. } if actual == &resource)
                );
                assert_eq!(f.qualification().unwrap().context(), cx.identity());
                assert_eq!(ss.query(&[]).unwrap_err(), f);
            }
            assert_eq!(ss.usage().scratch_records(), 0);
        }
    }
}

#[test]
fn discarded_fallback_text_capacity_is_cumulative_and_second_batch_query_is_atomic() {
    let (c, s, o, _) = gap_fixture(&[], Some("fallback text"));
    let cx = context(&c, &[(o, Some(true))]);
    let single = report(&c, &cx, &s, 0).usage();
    let mut ss = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let r = ss.query(&[request(&s), request(&s)]).unwrap();
    assert_eq!(
        r.usage().nested_output_records(),
        2 * single.nested_output_records()
    );
    assert!(r.usage().allocated_bytes() > single.allocated_bytes());
    let mut ss = session(
        &c,
        &cx,
        resource_limits(
            HtmlSemanticResource::NestedOutputRecords,
            single.nested_output_records(),
        ),
        0,
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
fn unavailable_fallback_value_keeps_already_observed_partial_buffer_cost() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select", &[("required", "")]);
    let gap = html(&mut b, &d, "div", &[]);
    let o = html(&mut b, &d, "option", &[]);
    let t = b.add_text(&d, "partial".into()).unwrap();
    let unavailable = html(&mut b, &d, "span", &[]);
    children(&mut b, &o, &[t, unavailable]);
    children(&mut b, &s, &[gap.clone(), o.clone()]);
    let c = b.finish().unwrap();
    let cx = context(&c, &[(c.subject_id(&o).unwrap(), Some(true))]);
    let r = report(&c, &cx, &c.subject_id(&s).unwrap(), 0);
    assert_eq!(
        missing(&r),
        &HtmlSemanticMissing::OrderedChildren(c.subject_id(&gap).unwrap())
    );
    assert!(r.usage().nested_output_records() >= 7);
    assert!(r.usage().allocated_bytes() > 0);
    assert_eq!(r.usage().scratch_records(), 0);
}

fn predecessor_fixture(
    gap_first: bool,
) -> (
    HtmlCapture,
    HtmlCaptureSubjectId,
    HtmlCaptureSubjectId,
    HtmlCaptureSubjectId,
) {
    let (mut b, d) = builder();
    let s = html(
        &mut b,
        &d,
        "select",
        &[("required", ""), ("size", "unread size")],
    );
    let gap = html(&mut b, &d, "div", &[]);
    let a = html(
        &mut b,
        &d,
        "option",
        &[("value", "unread predecessor value")],
    );
    let selected = html(&mut b, &d, "option", &[("value", "unread selected value")]);
    let later = html(&mut b, &d, "span", &[]);
    if gap_first {
        children(&mut b, &s, &[gap, a.clone(), selected.clone(), later]);
    } else {
        children(&mut b, &s, &[a.clone(), gap, selected.clone(), later]);
    }
    let c = b.finish().unwrap();
    let sid = c.subject_id(&s).unwrap();
    let aid = c.subject_id(&a).unwrap();
    let selected_id = c.subject_id(&selected).unwrap();
    (c, sid, aid, selected_id)
}

#[test]
fn predecessor_with_unknown_selectedness_proves_false_without_value_size_or_later_order() {
    for gap_first in [true, false] {
        let (c, s, a, selected) = predecessor_fixture(gap_first);
        let cx = context(&c, &[(a, None), (selected, Some(true))]);
        let r = report(&c, &cx, &s, 0);
        assert!(!known(&r));
        assert_eq!(r.usage().nested_output_records(), 0);
        assert_eq!(r.usage().scratch_records(), 0);
    }
}

#[test]
fn pruned_option_descendants_cannot_supply_a_predecessor_witness() {
    for local in ["select", "hr", "datalist"] {
        let (mut b, d) = builder();
        let s = html(&mut b, &d, "select", &[("required", "")]);
        let gap = html(&mut b, &d, "div", &[]);
        let pruned = html(&mut b, &d, local, &[]);
        let excluded = html(&mut b, &d, "option", &[]);
        let selected = html(&mut b, &d, "option", &[("value", "")]);
        children(&mut b, &pruned, &[excluded]);
        children(&mut b, &s, &[gap.clone(), pruned, selected.clone()]);
        let c = b.finish().unwrap();
        let cx = context(&c, &[(c.subject_id(&selected).unwrap(), Some(true))]);
        let r = report(&c, &cx, &c.subject_id(&s).unwrap(), 0);
        assert_eq!(
            missing(&r),
            &HtmlSemanticMissing::OrderedChildren(c.subject_id(&gap).unwrap())
        );
        assert_eq!(r.usage().scratch_records(), 0);
    }
}

#[test]
fn a_single_possibly_first_empty_option_remains_missing_across_unknown_order() {
    let (c, s, o, gap) = gap_fixture(&[("value", "")], None);
    let cx = context(&c, &[(o, Some(true))]);
    let r = report(&c, &cx, &s, 0);
    assert_eq!(missing(&r), &HtmlSemanticMissing::OrderedChildren(gap));
    assert_eq!(r.usage().nested_output_records(), 0);
    assert_eq!(r.usage().scratch_records(), 0);
}

#[test]
fn predecessor_proof_has_exact_and_one_short_physical_admission_and_sticky_cleanup() {
    let (c, s, a, selected) = predecessor_fixture(true);
    let cx = context(&c, &[(a, None), (selected, Some(true))]);
    let baseline = report(&c, &cx, &s, 0);
    assert!(!known(&baseline));
    let u = baseline.usage();
    for (resource, exact) in [
        (HtmlSemanticResource::Work, u.work()),
        (HtmlSemanticResource::OutputRecords, u.output_records()),
        (
            HtmlSemanticResource::ScratchRecords,
            u.peak_scratch_records(),
        ),
        (HtmlSemanticResource::AllocatedBytes, u.allocated_bytes()),
    ] {
        assert!(exact > 0);
        for cap in [exact, exact - 1] {
            let mut ss = session(
                &c,
                &cx,
                resource_limits(resource, cap),
                0,
                HtmlSemanticCancellation::new(),
            );
            let result = ss.query(&[request(&s)]);
            if cap == exact {
                assert!(!known(&result.unwrap()));
            } else {
                let failure = result.unwrap_err();
                assert!(
                    matches!(failure.kind(), HtmlSemanticFailureKind::LimitExceeded { resource: actual, .. } if actual == &resource)
                );
                assert_eq!(failure.qualification().unwrap().context(), cx.identity());
                assert_eq!(ss.query(&[]).unwrap_err(), failure);
            }
            assert_eq!(ss.usage().scratch_records(), 0);
        }
    }
}

#[test]
fn predecessor_queries_charge_each_traversal_and_late_work_failure_is_atomic() {
    let (c, s, a, selected) = predecessor_fixture(true);
    let cx = context(&c, &[(a, None), (selected, Some(true))]);
    let mut both = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let complete = both.query(&[request(&s), request(&s)]).unwrap();
    assert_eq!(complete.results().len(), 2);
    for result in complete.results() {
        assert!(matches!(
            result.fact(),
            HtmlSemanticFact::SelectValueMissing(HtmlSemanticValue::Known(false))
        ));
    }
    assert_eq!(complete.usage().nested_output_records(), 0);
    assert!(complete.usage().work() > report(&c, &cx, &s, 0).usage().work());
    let mut ss = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::Work, complete.usage().work() - 1),
        0,
        HtmlSemanticCancellation::new(),
    );
    let failure = ss.query(&[request(&s), request(&s)]).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            ..
        }
    ));
    assert_eq!(failure.qualification().unwrap().capture(), c.identity());
    assert_eq!(ss.query(&[]).unwrap_err(), failure);
    assert_eq!(ss.usage().scratch_records(), 0);
}
