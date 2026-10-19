use surgeist_html::*;
use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureError,
    HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlCaptureSubjectId, HtmlCapturedIsValue,
    HtmlCapturedShadowProperties, HtmlControlFacts, HtmlControlFactsError, HtmlControlFactsLimits,
    HtmlControlFlag, HtmlCustomControlClassification, HtmlFormAssociation,
    HtmlFormAssociationState, HtmlFormOwner, HtmlFormOwnerOrigin, HtmlInputControlState,
    HtmlOptionControlState, HtmlScriptingStatus, HtmlSemanticCancellation, HtmlSemanticContext,
    HtmlSemanticContextError, HtmlSemanticContextInput, HtmlSemanticFact, HtmlSemanticFailureKind,
    HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport,
    HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticSession, HtmlSemanticUnsupported,
    HtmlSemanticValue, HtmlTextareaControlState,
};

#[test]
fn context_join_resumes_the_constructor_validation_budget() {
    let (mut b, d) = builder();
    let input = element(&mut b, &d, "input", &[]);
    b.establish_children(&input, &[]).unwrap();
    b.establish_children(&d, std::slice::from_ref(&input))
        .unwrap();
    let c = b.finish().unwrap();
    let facts = HtmlControlFacts::new(
        &c,
        c.host_facts_revision().clone(),
        vec![HtmlFormAssociation::new(
            c.subject_id(&input).unwrap(),
            HtmlFormAssociationState::ResetAgainstCapture,
        )],
        vec![],
        vec![],
        vec![],
        HtmlControlFactsLimits::new(1, 1000),
    )
    .unwrap();
    let error = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_controls(facts),
        0,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        HtmlSemanticContextError::ControlsFacts(HtmlControlFactsError::ValidationWork {
            limit: 1000,
            ..
        })
    ));
}

#[test]
fn context_rejects_false_custom_association_but_preserves_native_membership() {
    let (mut b, d) = builder();
    let custom = element(&mut b, &d, "x-control", &[]);
    let native = element(&mut b, &d, "input", &[]);
    b.establish_children(&custom, &[]).unwrap();
    b.establish_children(&native, &[]).unwrap();
    b.establish_children(&d, &[custom.clone(), native.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let custom_id = c.subject_id(&custom).unwrap();
    let native_id = c.subject_id(&native).unwrap();
    let rejected = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![HtmlCustomControlClassification::new(
            custom_id.clone(),
            false,
        )])
        .with_controls(reset(&c, &[&custom])),
        1,
    )
    .unwrap_err();
    assert_eq!(
        rejected,
        HtmlSemanticContextError::ContradictoryCustomControlAssociation(custom_id)
    );
    let cx = context(
        &c,
        Some(reset(&c, &[&native])),
        vec![HtmlCustomControlClassification::new(native_id, false)],
    );
    assert_eq!(
        owner(&query(&c, &cx, &native, HtmlSemanticQuery::FormOwner)).owner(),
        None
    );
}

fn builder() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(1000, 10000, 1000, 100000, 1000, 0, 0, 16_777_216),
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
    name: &str,
    attrs: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    expanded(
        b,
        d,
        Namespace::html(),
        name,
        OrderedAttributes::new(attrs.iter().map(|(name, value)| {
            Attribute::new(QualifiedName::new(Namespace::none(), None, *name), *value)
        }))
        .unwrap(),
    )
}
fn expanded(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    ns: Namespace,
    name: &str,
    attrs: OrderedAttributes,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(ns, None, name),
        attrs,
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn facts(
    c: &HtmlCapture,
    associations: Vec<HtmlFormAssociation>,
    inputs: Vec<HtmlInputControlState>,
    options: Vec<HtmlOptionControlState>,
    textareas: Vec<HtmlTextareaControlState>,
) -> HtmlControlFacts {
    HtmlControlFacts::new(
        c,
        c.host_facts_revision().clone(),
        associations,
        inputs,
        options,
        textareas,
        HtmlControlFactsLimits::new(100, 1000000),
    )
    .unwrap()
}
fn reset(c: &HtmlCapture, nodes: &[&HtmlCaptureBuilderSubject]) -> HtmlControlFacts {
    facts(
        c,
        nodes
            .iter()
            .map(|n| {
                HtmlFormAssociation::new(
                    c.subject_id(n).unwrap(),
                    HtmlFormAssociationState::ResetAgainstCapture,
                )
            })
            .collect(),
        vec![],
        vec![],
        vec![],
    )
}
fn context(
    c: &HtmlCapture,
    controls: Option<HtmlControlFacts>,
    custom: Vec<HtmlCustomControlClassification>,
) -> HtmlSemanticContext {
    let max = custom.capacity();
    let mut input = HtmlSemanticContextInput::new(custom);
    if let Some(controls) = controls {
        input = input.with_controls(controls);
    }
    HtmlSemanticContext::new(c, c.host_facts_revision().clone(), input, max).unwrap()
}
fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(1000000, 100, 1000, 1000, 1000000, 1000)
}
fn query(
    c: &HtmlCapture,
    cx: &HtmlSemanticContext,
    e: &HtmlCaptureBuilderSubject,
    q: HtmlSemanticQuery,
) -> HtmlSemanticReport {
    HtmlSemanticSession::new(
        c,
        cx,
        HtmlValueInterpreter::new(10000),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[HtmlSemanticRequest::new(c.subject_id(e).unwrap(), q)])
    .unwrap()
}
fn owner(r: &HtmlSemanticReport) -> &HtmlFormOwner {
    match r.results()[0].fact() {
        HtmlSemanticFact::FormOwner(HtmlSemanticValue::Known(v)) => v,
        other => panic!("expected owner, got {other:?}"),
    }
}
fn labeled(r: &HtmlSemanticReport) -> Option<&HtmlCaptureSubjectId> {
    match r.results()[0].fact() {
        HtmlSemanticFact::LabeledControl(HtmlSemanticValue::Known(v)) => v.as_ref(),
        other => panic!("expected label, got {other:?}"),
    }
}
fn flag(r: &HtmlSemanticReport) -> &HtmlSemanticValue<bool> {
    match r.results()[0].fact() {
        HtmlSemanticFact::CurrentCheckedness(v)
        | HtmlSemanticFact::CurrentSelectedness(v)
        | HtmlSemanticFact::DirtyValueFlag(v)
        | HtmlSemanticFact::DirtyCheckednessFlag(v)
        | HtmlSemanticFact::OptionDirtiness(v)
        | HtmlSemanticFact::Indeterminateness(v) => v,
        other => panic!("expected current flag, got {other:?}"),
    }
}
fn shadow(
    b: &mut HtmlCaptureBuilder,
    document: &HtmlCaptureBuilderSubject,
    host: &HtmlCaptureBuilderSubject,
) -> HtmlCaptureBuilderSubject {
    b.add_shadow_root(
        document,
        host,
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
}

#[test]
fn reset_finds_nearest_nested_form_and_current_pointer_preserves_nonancestor_history() {
    let (mut b, d) = builder();
    let outside = element(&mut b, &d, "form", &[]);
    let outer = element(&mut b, &d, "form", &[]);
    let inner = element(&mut b, &d, "form", &[]);
    let e = element(&mut b, &d, "input", &[]);
    b.establish_children(&inner, std::slice::from_ref(&e))
        .unwrap();
    b.establish_children(&outer, std::slice::from_ref(&inner))
        .unwrap();
    b.establish_children(&d, &[outside.clone(), outer]).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, Some(reset(&c, &[&e])), vec![]);
    let report = query(&c, &cx, &e, HtmlSemanticQuery::FormOwner);
    assert_eq!(owner(&report).owner(), Some(&c.subject_id(&inner).unwrap()));
    assert_eq!(
        owner(&report).origin(),
        HtmlFormOwnerOrigin::ResetAgainstCapture
    );
    let established = HtmlFormAssociation::new(
        c.subject_id(&e).unwrap(),
        HtmlFormAssociationState::EstablishedCurrent(Some(c.subject_id(&outside).unwrap())),
    );
    let cx = context(
        &c,
        Some(facts(&c, vec![established], vec![], vec![], vec![])),
        vec![],
    );
    let report = query(&c, &cx, &e, HtmlSemanticQuery::FormOwner);
    assert_eq!(
        owner(&report).owner(),
        Some(&c.subject_id(&outside).unwrap())
    );
    assert_eq!(
        owner(&report).origin(),
        HtmlFormOwnerOrigin::EstablishedCurrent
    );
}

#[test]
fn current_none_is_evidence_and_absent_history_does_not_guess_ancestor() {
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", &[]);
    let e = element(&mut b, &d, "input", &[]);
    b.establish_children(&form, std::slice::from_ref(&e))
        .unwrap();
    let c = b.finish().unwrap();
    let absent = query(
        &c,
        &context(&c, None, vec![]),
        &e,
        HtmlSemanticQuery::FormOwner,
    );
    assert!(matches!(
        absent.results()[0].fact(),
        HtmlSemanticFact::FormOwner(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::FormAssociation(_)
        ))
    ));
    let known = HtmlFormAssociation::new(
        c.subject_id(&e).unwrap(),
        HtmlFormAssociationState::EstablishedCurrent(None),
    );
    let report = query(
        &c,
        &context(
            &c,
            Some(facts(&c, vec![known], vec![], vec![], vec![])),
            vec![],
        ),
        &e,
        HtmlSemanticQuery::FormOwner,
    );
    assert_eq!(owner(&report).owner(), None);
}

#[test]
fn connected_explicit_form_uses_first_id_and_nonform_blocks_later_form_without_ancestor_fallback() {
    let (mut b, d) = builder();
    let blocker = element(&mut b, &d, "div", &[("id", "target")]);
    let target = element(&mut b, &d, "form", &[("id", "target")]);
    let ancestor = element(&mut b, &d, "form", &[]);
    let e = element(&mut b, &d, "input", &[("form", "target")]);
    b.establish_children(&blocker, &[]).unwrap();
    b.establish_children(&ancestor, std::slice::from_ref(&e))
        .unwrap();
    b.establish_children(&d, &[blocker, target, ancestor])
        .unwrap();
    let c = b.finish().unwrap();
    let report = query(
        &c,
        &context(&c, Some(reset(&c, &[&e])), vec![]),
        &e,
        HtmlSemanticQuery::FormOwner,
    );
    assert_eq!(owner(&report).owner(), None);
}

#[test]
fn connected_explicit_form_can_be_nonancestor_and_stops_at_proved_prefix() {
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", &[("id", "target")]);
    let e = element(&mut b, &d, "input", &[("form", "target")]);
    let unknown = element(&mut b, &d, "section", &[]);
    b.establish_children(&d, &[form.clone(), e.clone(), unknown])
        .unwrap();
    let c = b.finish().unwrap();
    let r = query(
        &c,
        &context(&c, Some(reset(&c, &[&e])), vec![]),
        &e,
        HtmlSemanticQuery::FormOwner,
    );
    assert_eq!(owner(&r).owner(), Some(&c.subject_id(&form).unwrap()));
}

#[test]
fn disconnected_form_attribute_uses_nearest_ordinary_ancestor() {
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", &[]);
    let e = element(&mut b, &d, "input", &[("form", "unresolved")]);
    b.establish_children(&form, std::slice::from_ref(&e))
        .unwrap();
    b.establish_root(&form).unwrap();
    let c = b.finish().unwrap();
    let r = query(
        &c,
        &context(&c, Some(reset(&c, &[&e])), vec![]),
        &e,
        HtmlSemanticQuery::FormOwner,
    );
    assert_eq!(owner(&r).owner(), Some(&c.subject_id(&form).unwrap()));
}

#[test]
fn img_is_form_associated_but_not_listed_and_ignores_form_attribute() {
    let (mut b, d) = builder();
    let other = element(&mut b, &d, "form", &[("id", "other")]);
    let form = element(&mut b, &d, "form", &[]);
    let img = element(&mut b, &d, "img", &[("form", "other")]);
    b.establish_children(&form, std::slice::from_ref(&img))
        .unwrap();
    b.establish_children(&d, &[other, form.clone()]).unwrap();
    let c = b.finish().unwrap();
    let r = query(
        &c,
        &context(&c, Some(reset(&c, &[&img])), vec![]),
        &img,
        HtmlSemanticQuery::FormOwner,
    );
    assert_eq!(owner(&r).owner(), Some(&c.subject_id(&form).unwrap()));
}

#[test]
fn shadow_connectedness_crosses_host_but_id_lookup_and_ancestor_forms_do_not() {
    let (mut b, d) = builder();
    let outer = element(&mut b, &d, "form", &[("id", "outer")]);
    let host = element(&mut b, &d, "div", &[]);
    b.establish_children(&outer, std::slice::from_ref(&host))
        .unwrap();
    b.establish_children(&d, &[outer]).unwrap();
    let root = shadow(&mut b, &d, &host);
    let e = element(&mut b, &d, "input", &[("form", "outer")]);
    let no_attr = element(&mut b, &d, "input", &[]);
    b.establish_children(&e, &[]).unwrap();
    b.establish_children(&no_attr, &[]).unwrap();
    b.establish_children(&root, &[e.clone(), no_attr.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, Some(reset(&c, &[&e, &no_attr])), vec![]);
    for e in [&e, &no_attr] {
        assert_eq!(
            owner(&query(&c, &cx, e, HtmlSemanticQuery::FormOwner)).owner(),
            None
        );
    }
}

#[test]
fn shadow_explicit_form_targets_same_shadow_tree_when_host_is_connected() {
    let (mut b, d) = builder();
    let host = element(&mut b, &d, "div", &[]);
    b.establish_children(&d, std::slice::from_ref(&host))
        .unwrap();
    let root = shadow(&mut b, &d, &host);
    let target = element(&mut b, &d, "form", &[("id", "x")]);
    let e = element(&mut b, &d, "input", &[("form", "x")]);
    b.establish_children(&root, &[target.clone(), e.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let r = query(
        &c,
        &context(&c, Some(reset(&c, &[&e])), vec![]),
        &e,
        HtmlSemanticQuery::FormOwner,
    );
    assert_eq!(owner(&r).owner(), Some(&c.subject_id(&target).unwrap()));
}

#[test]
fn explicit_empty_form_leaves_connected_control_ownerless() {
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", &[]);
    let e = element(&mut b, &d, "input", &[("form", "")]);
    b.establish_children(&form, std::slice::from_ref(&e))
        .unwrap();
    b.establish_children(&d, &[form]).unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        owner(&query(
            &c,
            &context(&c, Some(reset(&c, &[&e])), vec![]),
            &e,
            HtmlSemanticQuery::FormOwner
        ))
        .owner(),
        None
    );
}

#[test]
fn present_empty_for_never_uses_descendant_and_nonlabelable_first_id_blocks() {
    let (mut b, d) = builder();
    let label = element(&mut b, &d, "label", &[("for", "")]);
    let input = element(&mut b, &d, "input", &[]);
    b.establish_children(&label, &[input]).unwrap();
    let blocked = element(&mut b, &d, "label", &[("for", "id")]);
    let div = element(&mut b, &d, "div", &[("id", "id")]);
    let target = element(&mut b, &d, "input", &[("id", "id")]);
    b.establish_children(&d, &[div, blocked.clone(), target, label.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, None, vec![]);
    assert_eq!(
        labeled(&query(&c, &cx, &label, HtmlSemanticQuery::LabeledControl)),
        None
    );
    assert_eq!(
        labeled(&query(&c, &cx, &blocked, HtmlSemanticQuery::LabeledControl)),
        None
    );
}

#[test]
fn missing_for_finds_first_labelable_native_descendant_including_meter_and_progress() {
    let (mut b, d) = builder();
    let label = element(&mut b, &d, "label", &[]);
    let hidden = element(&mut b, &d, "input", &[("type", "hidden")]);
    b.establish_children(&hidden, &[]).unwrap();
    let meter = element(&mut b, &d, "meter", &[]);
    let progress = element(&mut b, &d, "progress", &[]);
    b.establish_children(&label, &[hidden, meter.clone(), progress])
        .unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        labeled(&query(
            &c,
            &context(&c, None, vec![]),
            &label,
            HtmlSemanticQuery::LabeledControl
        )),
        Some(&c.subject_id(&meter).unwrap())
    );
}

#[test]
fn label_id_is_exact_case_and_utf16_and_first_match_is_tested_before_later_controls() {
    let (mut b, d) = builder();
    let exact = DocumentString::from_utf16(vec![0xd800, 0, 65]);
    let label = expanded(
        &mut b,
        &d,
        Namespace::html(),
        "label",
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::none(), None, "for"),
            exact.clone(),
        )])
        .unwrap(),
    );
    let target = expanded(
        &mut b,
        &d,
        Namespace::html(),
        "input",
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::none(), None, "id"),
            exact,
        )])
        .unwrap(),
    );
    let later = element(&mut b, &d, "div", &[]);
    b.establish_children(&label, &[]).unwrap();
    b.establish_children(&d, &[label.clone(), target.clone(), later])
        .unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        labeled(&query(
            &c,
            &context(&c, None, vec![]),
            &label,
            HtmlSemanticQuery::LabeledControl
        )),
        Some(&c.subject_id(&target).unwrap())
    );
}

#[test]
fn label_in_shadow_tree_does_not_target_light_dom_id_or_follow_assigned_nodes() {
    let (mut b, d) = builder();
    let host = element(&mut b, &d, "div", &[]);
    let target = element(&mut b, &d, "input", &[("id", "x")]);
    b.establish_children(&host, &[target]).unwrap();
    b.establish_children(&d, std::slice::from_ref(&host))
        .unwrap();
    let root = shadow(&mut b, &d, &host);
    let label = element(&mut b, &d, "label", &[("for", "x")]);
    b.establish_children(&label, &[]).unwrap();
    b.establish_children(&root, std::slice::from_ref(&label))
        .unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        labeled(&query(
            &c,
            &context(&c, None, vec![]),
            &label,
            HtmlSemanticQuery::LabeledControl
        )),
        None
    );
}

#[test]
fn custom_classification_is_actual_form_associated_state_for_owner_and_labels() {
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", &[]);
    let label = element(&mut b, &d, "label", &[]);
    let custom = element(&mut b, &d, "x-control", &[]);
    b.establish_children(&custom, &[]).unwrap();
    b.establish_children(&label, std::slice::from_ref(&custom))
        .unwrap();
    b.establish_children(&form, std::slice::from_ref(&label))
        .unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&custom).unwrap();
    let cx = context(
        &c,
        Some(reset(&c, &[&custom])),
        vec![HtmlCustomControlClassification::new(id.clone(), true)],
    );
    assert_eq!(
        owner(&query(&c, &cx, &custom, HtmlSemanticQuery::FormOwner)).owner(),
        Some(&c.subject_id(&form).unwrap())
    );
    assert_eq!(
        labeled(&query(&c, &cx, &label, HtmlSemanticQuery::LabeledControl)),
        Some(&id)
    );
    let missing = query(
        &c,
        &context(&c, None, vec![]),
        &label,
        HtmlSemanticQuery::LabeledControl,
    );
    assert!(matches!(
        missing.results()[0].fact(),
        HtmlSemanticFact::LabeledControl(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::CustomControlClassification(_)
        ))
    ));
}

#[test]
fn earlier_unavailable_prefix_remains_missing_instead_of_skipping_to_later_id() {
    let (mut b, d) = builder();
    let earlier = element(&mut b, &d, "section", &[]);
    let target = element(&mut b, &d, "input", &[("id", "x")]);
    let label = element(&mut b, &d, "label", &[("for", "x")]);
    b.establish_children(&d, &[earlier, target, label.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let report = query(
        &c,
        &context(&c, None, vec![]),
        &label,
        HtmlSemanticQuery::LabeledControl,
    );
    assert!(matches!(
        report.results()[0].fact(),
        HtmlSemanticFact::LabeledControl(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::OrderedChildren(_)
        ))
    ));
}

#[test]
fn markup_checked_selected_do_not_override_established_current_flags() {
    let (mut b, d) = builder();
    let input = element(
        &mut b,
        &d,
        "input",
        &[("type", "checkbox"), ("checked", "")],
    );
    let option = element(&mut b, &d, "option", &[("selected", "")]);
    let textarea = element(&mut b, &d, "textarea", &[]);
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![],
            vec![HtmlInputControlState::new(
                c.subject_id(&input).unwrap(),
                Some(false),
                Some(true),
                Some(true),
                Some(true),
            )],
            vec![HtmlOptionControlState::new(
                c.subject_id(&option).unwrap(),
                Some(false),
                Some(true),
            )],
            vec![HtmlTextareaControlState::new(
                c.subject_id(&textarea).unwrap(),
                Some(true),
            )],
        )),
        vec![],
    );
    for (e, q, expected) in [
        (&input, HtmlSemanticQuery::CurrentCheckedness, false),
        (&input, HtmlSemanticQuery::DirtyCheckednessFlag, true),
        (&input, HtmlSemanticQuery::DirtyValueFlag, true),
        (&input, HtmlSemanticQuery::Indeterminateness, true),
        (&option, HtmlSemanticQuery::CurrentSelectedness, false),
        (&option, HtmlSemanticQuery::OptionDirtiness, true),
        (&textarea, HtmlSemanticQuery::DirtyValueFlag, true),
    ] {
        assert_eq!(
            flag(&query(&c, &cx, e, q)),
            &HtmlSemanticValue::Known(expected)
        );
    }
}

#[test]
fn individual_unavailable_slot_does_not_erase_other_known_slots() {
    let (mut b, d) = builder();
    let input = element(&mut b, &d, "input", &[("checked", "")]);
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![],
            vec![HtmlInputControlState::new(
                c.subject_id(&input).unwrap(),
                None,
                Some(false),
                None,
                Some(false),
            )],
            vec![],
            vec![],
        )),
        vec![],
    );
    assert!(matches!(
        flag(&query(
            &c,
            &cx,
            &input,
            HtmlSemanticQuery::CurrentCheckedness
        )),
        HtmlSemanticValue::Missing(HtmlSemanticMissing::ControlState {
            flag: HtmlControlFlag::Checkedness,
            ..
        })
    ));
    assert_eq!(
        flag(&query(
            &c,
            &cx,
            &input,
            HtmlSemanticQuery::DirtyCheckednessFlag
        )),
        &HtmlSemanticValue::Known(false)
    );
    assert!(matches!(
        flag(&query(
            &c,
            &context(&c, None, vec![]),
            &input,
            HtmlSemanticQuery::CurrentCheckedness
        )),
        HtmlSemanticValue::Missing(_)
    ));
}

#[test]
fn internal_input_checkedness_exists_even_when_text_ui_does_not_use_it() {
    let (mut b, d) = builder();
    let input = element(&mut b, &d, "input", &[("type", "text")]);
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![],
            vec![HtmlInputControlState::new(
                c.subject_id(&input).unwrap(),
                Some(true),
                Some(false),
                Some(false),
                Some(false),
            )],
            vec![],
            vec![],
        )),
        vec![],
    );
    assert_eq!(
        flag(&query(
            &c,
            &cx,
            &input,
            HtmlSemanticQuery::CurrentCheckedness
        )),
        &HtmlSemanticValue::Known(true)
    );
    assert_eq!(
        flag(&query(
            &c,
            &cx,
            &input,
            HtmlSemanticQuery::CurrentSelectedness
        )),
        &HtmlSemanticValue::Inapplicable
    );
}

#[test]
fn facts_constructor_checks_kinds_foreign_ids_targets_host_and_combined_capacity() {
    let (mut b, d) = builder();
    let input = element(&mut b, &d, "input", &[]);
    let div = element(&mut b, &d, "div", &[]);
    let c = b.finish().unwrap();
    let caps = HtmlControlFactsLimits::new(100, 100000);
    assert!(matches!(
        HtmlControlFacts::new(
            &c,
            HtmlHostFactsRevision::fresh(),
            vec![],
            vec![],
            vec![],
            vec![],
            caps
        ),
        Err(HtmlControlFactsError::HostRevisionMismatch)
    ));
    let invalid = HtmlFormAssociation::new(
        c.subject_id(&input).unwrap(),
        HtmlFormAssociationState::EstablishedCurrent(Some(c.subject_id(&div).unwrap())),
    );
    assert!(matches!(
        HtmlControlFacts::new(
            &c,
            c.host_facts_revision().clone(),
            vec![invalid],
            vec![],
            vec![],
            vec![],
            caps
        ),
        Err(HtmlControlFactsError::InvalidFormOwner(_))
    ));
    let wrong = HtmlOptionControlState::new(c.subject_id(&input).unwrap(), None, None);
    assert!(matches!(
        HtmlControlFacts::new(
            &c,
            c.host_facts_revision().clone(),
            vec![],
            vec![],
            vec![wrong],
            vec![],
            caps
        ),
        Err(HtmlControlFactsError::InapplicableSubject(_))
    ));
    let (b2, d2) = builder();
    let other = b2.finish().unwrap();
    let foreign = HtmlFormAssociation::new(
        other.subject_id(&d2).unwrap(),
        HtmlFormAssociationState::Unavailable,
    );
    assert!(matches!(
        HtmlControlFacts::new(
            &c,
            c.host_facts_revision().clone(),
            vec![foreign],
            vec![],
            vec![],
            vec![],
            caps
        ),
        Err(HtmlControlFactsError::Capture(
            HtmlCaptureError::ForeignCapture
        ))
    ));
    let mut inputs = Vec::with_capacity(3);
    inputs.push(HtmlInputControlState::new(
        c.subject_id(&input).unwrap(),
        None,
        None,
        None,
        None,
    ));
    assert!(matches!(
        HtmlControlFacts::new(
            &c,
            c.host_facts_revision().clone(),
            vec![],
            inputs,
            vec![],
            vec![],
            HtmlControlFactsLimits::new(2, 100000)
        ),
        Err(HtmlControlFactsError::RecordCapacity {
            limit: 2,
            required: 3
        })
    ));
}

#[test]
fn duplicate_records_and_exhausted_validation_work_reject_before_context_creation() {
    let (mut b, d) = builder();
    let input = element(&mut b, &d, "input", &[]);
    let c = b.finish().unwrap();
    let one = HtmlInputControlState::new(c.subject_id(&input).unwrap(), None, None, None, None);
    assert!(matches!(
        HtmlControlFacts::new(
            &c,
            c.host_facts_revision().clone(),
            vec![],
            vec![one.clone(), one.clone()],
            vec![],
            vec![],
            HtmlControlFactsLimits::new(2, 100000)
        ),
        Err(HtmlControlFactsError::DuplicateSubject(_))
    ));
    assert!(matches!(
        HtmlControlFacts::new(
            &c,
            c.host_facts_revision().clone(),
            vec![],
            vec![one],
            vec![],
            vec![],
            HtmlControlFactsLimits::new(1, 0)
        ),
        Err(HtmlControlFactsError::ValidationWork { limit: 0, .. })
    ));
}

#[test]
fn work_scratch_bytes_cancel_and_foreign_requests_preserve_qualification_and_atomic_batches() {
    let (mut b, d) = builder();
    let label = element(&mut b, &d, "label", &[]);
    let wrapper = element(&mut b, &d, "div", &[]);
    let input = element(&mut b, &d, "input", &[]);
    b.establish_children(&wrapper, std::slice::from_ref(&input))
        .unwrap();
    b.establish_children(&label, &[wrapper]).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, None, vec![]);
    let request = HtmlSemanticRequest::new(
        c.subject_id(&label).unwrap(),
        HtmlSemanticQuery::LabeledControl,
    );
    let make = |l, cancel| {
        HtmlSemanticSession::new(&c, &cx, HtmlValueInterpreter::new(100), l, cancel).unwrap()
    };
    let preserved = make(limits(), HtmlSemanticCancellation::new())
        .query(std::slice::from_ref(&request))
        .unwrap();
    let work = preserved.usage().work();
    assert_eq!(preserved.usage().scratch_records(), 0);
    let mut short = make(
        HtmlSemanticLimits::new(work - 1, 100, 1000, 1000, 1000000, 1000),
        HtmlSemanticCancellation::new(),
    );
    let failure = short.query(std::slice::from_ref(&request)).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            ..
        }
    ));
    assert_eq!(short.usage().scratch_records(), 0);
    assert_eq!(short.query(&[]).unwrap_err(), failure);
    assert_eq!(failure.qualification().unwrap().capture(), c.identity());
    assert_eq!(
        labeled(
            &make(
                HtmlSemanticLimits::new(work, 100, 1000, 1000, 1000000, 1000),
                HtmlSemanticCancellation::new()
            )
            .query(std::slice::from_ref(&request))
            .unwrap()
        ),
        Some(&c.subject_id(&input).unwrap())
    );
    for (l, res) in [
        (
            HtmlSemanticLimits::new(1000000, 100, 0, 1000, 1000000, 1000),
            HtmlSemanticResource::ScratchRecords,
        ),
        (
            HtmlSemanticLimits::new(1000000, 100, 1000, 1000, 0, 1000),
            HtmlSemanticResource::AllocatedBytes,
        ),
    ] {
        let mut session = make(l, HtmlSemanticCancellation::new());
        let failure = session.query(std::slice::from_ref(&request)).unwrap_err();
        assert!(
            matches!(failure.kind(),HtmlSemanticFailureKind::LimitExceeded{resource,..} if *resource==res)
        );
        assert_eq!(session.usage().scratch_records(), 0);
    }
    let cancel = HtmlSemanticCancellation::new();
    let mut cancelled = make(limits(), cancel.clone());
    cancel.cancel();
    let failure = cancelled.query(std::slice::from_ref(&request)).unwrap_err();
    assert_eq!(failure.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(cancelled.query(&[]).unwrap_err(), failure);
    assert_eq!(labeled(&preserved), Some(&c.subject_id(&input).unwrap()));
    let (b2, d2) = builder();
    let other = b2.finish().unwrap();
    let foreign = HtmlSemanticRequest::new(
        other.subject_id(&d2).unwrap(),
        HtmlSemanticQuery::LabeledControl,
    );
    let failure = make(limits(), HtmlSemanticCancellation::new())
        .query(&[request, foreign])
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Capture(HtmlCaptureError::ForeignCapture)
    ));
}

#[test]
fn actual_owner_pointer_can_be_read_with_unavailable_ancestry_but_proved_cross_tree_is_rejected() {
    let (mut b, d) = builder();
    let input = element(&mut b, &d, "input", &[]);
    let form = element(&mut b, &d, "form", &[]);
    let c = b.finish().unwrap();
    let established = HtmlFormAssociation::new(
        c.subject_id(&input).unwrap(),
        HtmlFormAssociationState::EstablishedCurrent(Some(c.subject_id(&form).unwrap())),
    );
    let r = query(
        &c,
        &context(
            &c,
            Some(facts(&c, vec![established], vec![], vec![], vec![])),
            vec![],
        ),
        &input,
        HtmlSemanticQuery::FormOwner,
    );
    assert_eq!(owner(&r).owner(), Some(&c.subject_id(&form).unwrap()));
    let (mut b, d) = builder();
    let input = element(&mut b, &d, "input", &[]);
    let form = element(&mut b, &d, "form", &[]);
    b.establish_root(&input).unwrap();
    b.establish_root(&form).unwrap();
    let c = b.finish().unwrap();
    let established = HtmlFormAssociation::new(
        c.subject_id(&input).unwrap(),
        HtmlFormAssociationState::EstablishedCurrent(Some(c.subject_id(&form).unwrap())),
    );
    assert!(matches!(
        HtmlControlFacts::new(
            &c,
            c.host_facts_revision().clone(),
            vec![established],
            vec![],
            vec![],
            vec![],
            HtmlControlFactsLimits::new(1, 100000)
        ),
        Err(HtmlControlFactsError::CrossTreeFormOwner(_))
    ));
}

#[test]
fn false_custom_classification_does_not_create_a_labelable_match() {
    let (mut b, d) = builder();
    let label = element(&mut b, &d, "label", &[]);
    let custom = element(&mut b, &d, "x-control", &[]);
    b.establish_children(&custom, &[]).unwrap();
    let progress = element(&mut b, &d, "progress", &[]);
    b.establish_children(&label, &[custom.clone(), progress.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        None,
        vec![HtmlCustomControlClassification::new(
            c.subject_id(&custom).unwrap(),
            false,
        )],
    );
    assert_eq!(
        labeled(&query(&c, &cx, &label, HtmlSemanticQuery::LabeledControl)),
        Some(&c.subject_id(&progress).unwrap())
    );
}

#[test]
fn for_id_comparison_is_case_sensitive_and_namespaced_for_is_ignored() {
    let (mut b, d) = builder();
    let upper = element(&mut b, &d, "input", &[("id", "X")]);
    b.establish_children(&upper, &[]).unwrap();
    let lower = element(&mut b, &d, "input", &[("id", "x")]);
    let label = element(&mut b, &d, "label", &[("for", "x")]);
    b.establish_children(&label, &[]).unwrap();
    b.establish_children(&d, &[upper, lower.clone(), label.clone()])
        .unwrap();
    let namespaced = expanded(
        &mut b,
        &d,
        Namespace::html(),
        "label",
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::from_uri("urn:other"), Some("x".into()), "for"),
            "unmatched",
        )])
        .unwrap(),
    );
    let child = element(&mut b, &d, "button", &[]);
    b.establish_children(&namespaced, std::slice::from_ref(&child))
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, None, vec![]);
    assert_eq!(
        labeled(&query(&c, &cx, &label, HtmlSemanticQuery::LabeledControl)),
        Some(&c.subject_id(&lower).unwrap())
    );
    assert_eq!(
        labeled(&query(
            &c,
            &cx,
            &namespaced,
            HtmlSemanticQuery::LabeledControl
        )),
        Some(&c.subject_id(&child).unwrap())
    );
}

#[test]
fn inapplicable_native_and_foreign_subjects_are_distinct_from_missing_current_state() {
    let (mut b, d) = builder();
    let div = element(&mut b, &d, "div", &[]);
    let foreign = expanded(
        &mut b,
        &d,
        Namespace::svg(),
        "input",
        OrderedAttributes::new([]).unwrap(),
    );
    let option = element(&mut b, &d, "option", &[]);
    let c = b.finish().unwrap();
    let cx = context(&c, None, vec![]);
    assert!(matches!(
        query(&c, &cx, &div, HtmlSemanticQuery::FormOwner).results()[0].fact(),
        HtmlSemanticFact::FormOwner(HtmlSemanticValue::Inapplicable)
    ));
    assert_eq!(
        flag(&query(
            &c,
            &cx,
            &foreign,
            HtmlSemanticQuery::CurrentCheckedness
        )),
        &HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
    );
    assert_eq!(
        flag(&query(&c, &cx, &option, HtmlSemanticQuery::DirtyValueFlag)),
        &HtmlSemanticValue::Inapplicable
    );
    assert!(matches!(
        flag(&query(
            &c,
            &cx,
            &option,
            HtmlSemanticQuery::CurrentSelectedness
        )),
        HtmlSemanticValue::Missing(_)
    ));
}

#[test]
fn established_unavailable_association_and_incomplete_reset_parent_remain_missing() {
    let (mut b, d) = builder();
    let input = element(&mut b, &d, "input", &[]);
    let c = b.finish().unwrap();
    let unavailable = HtmlFormAssociation::new(
        c.subject_id(&input).unwrap(),
        HtmlFormAssociationState::Unavailable,
    );
    let cx = context(
        &c,
        Some(facts(&c, vec![unavailable], vec![], vec![], vec![])),
        vec![],
    );
    assert!(matches!(
        query(&c, &cx, &input, HtmlSemanticQuery::FormOwner).results()[0].fact(),
        HtmlSemanticFact::FormOwner(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::FormAssociation(_)
        ))
    ));
    let cx = context(&c, Some(reset(&c, &[&input])), vec![]);
    assert!(matches!(
        query(&c, &cx, &input, HtmlSemanticQuery::FormOwner).results()[0].fact(),
        HtmlSemanticFact::FormOwner(HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(_)))
    ));
}
