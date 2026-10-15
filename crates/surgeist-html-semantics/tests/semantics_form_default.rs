use surgeist_html::*;
use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureLimits,
    HtmlCaptureRegistryInput, HtmlCaptureSubjectId, HtmlCapturedIsValue,
    HtmlCapturedShadowProperties, HtmlControlFacts, HtmlControlFactsLimits, HtmlDefaultButton,
    HtmlFormAssociation, HtmlFormAssociationState, HtmlFormOwnerOrigin,
    HtmlImplicitSubmissionBlockingFields, HtmlImplicitSubmissionCondition, HtmlScriptingStatus,
    HtmlSemanticCancellation, HtmlSemanticContext, HtmlSemanticContextInput,
    HtmlSemanticDependencies, HtmlSemanticFact, HtmlSemanticFailureKind, HtmlSemanticLimits,
    HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport, HtmlSemanticRequest,
    HtmlSemanticResource, HtmlSemanticSession, HtmlSemanticUnsupported, HtmlSemanticValue,
};

fn attributes(values: &[(&str, &str)]) -> OrderedAttributes {
    OrderedAttributes::new(
        values
            .iter()
            .map(|(n, v)| Attribute::new(QualifiedName::new(Namespace::none(), None, *n), *v)),
    )
    .unwrap()
}
fn builder() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 1000, 100, 100000, 100, 0, 0),
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
    b.add_element(
        d,
        QualifiedName::new(Namespace::html(), None, local),
        attributes(attrs),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
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
// All controls are external siblings before the form; their association is
// supplied explicitly, not obtained from a nearest ancestor approximation.
fn flat(values: &[(&str, &[(&str, &str)])]) -> Fixture {
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", &[]);
    let mut h = vec![form];
    for (local, attrs) in values {
        h.push(element(&mut b, &d, local, attrs));
    }
    for handle in &h {
        b.establish_children(handle, &[]).unwrap();
    }
    let mut ordered = h[1..].to_vec();
    ordered.push(h[0].clone());
    b.establish_children(&d, &ordered).unwrap();
    finish(b, &h)
}
fn context(f: &Fixture, states: Vec<(usize, HtmlFormAssociationState)>) -> HtmlSemanticContext {
    let facts = HtmlControlFacts::new(
        &f.capture,
        f.capture.host_facts_revision().clone(),
        states
            .into_iter()
            .map(|(i, s)| HtmlFormAssociation::new(f.ids[i].clone(), s))
            .collect(),
        vec![],
        vec![],
        vec![],
        HtmlControlFactsLimits::new(100, 1000000),
    )
    .unwrap();
    HtmlSemanticContext::new(
        &f.capture,
        f.capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_controls(facts),
        0,
    )
    .unwrap()
}
fn owned(f: &Fixture, indices: &[usize]) -> HtmlSemanticContext {
    context(
        f,
        indices
            .iter()
            .map(|i| {
                (
                    *i,
                    HtmlFormAssociationState::EstablishedCurrent(Some(f.ids[0].clone())),
                )
            })
            .collect(),
    )
}
fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(10000000, 100, 1000, 0, 1000000, 0)
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
fn report(f: &Fixture, c: &HtmlSemanticContext, query: HtmlSemanticQuery) -> HtmlSemanticReport {
    session(f, c, limits(), 10000, HtmlSemanticCancellation::new())
        .query(&[HtmlSemanticRequest::new(f.ids[0].clone(), query)])
        .unwrap()
}
fn default(r: &HtmlSemanticReport) -> &HtmlSemanticValue<Option<HtmlDefaultButton>> {
    match r.results()[0].fact() {
        HtmlSemanticFact::FormDefaultButton(v) => v,
        v => panic!("unexpected {v:?}"),
    }
}
fn condition(r: &HtmlSemanticReport) -> &HtmlSemanticValue<HtmlImplicitSubmissionCondition> {
    match r.results()[0].fact() {
        HtmlSemanticFact::FormImplicitSubmissionCondition(v) => v,
        v => panic!("unexpected {v:?}"),
    }
}
fn blockers(r: &HtmlSemanticReport) -> HtmlImplicitSubmissionBlockingFields {
    match condition(r) {
        HtmlSemanticValue::Known(HtmlImplicitSubmissionCondition::NoButton { blocking_fields }) => {
            *blocking_fields
        }
        v => panic!("expected no-button condition: {v:?}"),
    }
}

#[test]
fn external_image_before_submit_is_default_even_disabled() {
    let f = flat(&[
        ("input", &[("type", "image"), ("disabled", "false")]),
        ("button", &[("type", "submit")]),
    ]);
    let c = owned(&f, &[1, 2]);
    let r = report(&f, &c, HtmlSemanticQuery::FormDefaultButton);
    let HtmlSemanticValue::Known(Some(button)) = default(&r) else {
        panic!("{r:?}");
    };
    assert_eq!(button.subject(), &f.ids[1]);
    assert_eq!(button.form_owner().owner(), Some(&f.ids[0]));
    assert_eq!(
        button.form_owner().origin(),
        HtmlFormOwnerOrigin::EstablishedCurrent
    );
    let r = report(&f, &c, HtmlSemanticQuery::FormImplicitSubmissionCondition);
    assert!(
        matches!(condition(&r), HtmlSemanticValue::Known(HtmlImplicitSubmissionCondition::DefaultButton(b)) if b.subject()==&f.ids[1])
    );
}
#[test]
fn descendant_precedes_external_and_current_owner_overrides_ancestry() {
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", &[]);
    let nested = element(&mut b, &d, "form", &[]);
    let first = element(&mut b, &d, "button", &[("type", "submit")]);
    let external = element(&mut b, &d, "input", &[("type", "submit")]);
    b.establish_children(&first, &[]).unwrap();
    b.establish_children(&external, &[]).unwrap();
    b.establish_children(&nested, std::slice::from_ref(&first))
        .unwrap();
    b.establish_children(&form, std::slice::from_ref(&nested))
        .unwrap();
    b.establish_children(&d, &[form.clone(), external.clone()])
        .unwrap();
    let f = finish(b, &[form, first, external, nested]);
    let c = owned(&f, &[1, 2]);
    let r = report(&f, &c, HtmlSemanticQuery::FormDefaultButton);
    assert!(matches!(default(&r),HtmlSemanticValue::Known(Some(v)) if v.subject()==&f.ids[1]));
    let c = context(
        &f,
        vec![
            (
                1,
                HtmlFormAssociationState::EstablishedCurrent(Some(f.ids[3].clone())),
            ),
            (
                2,
                HtmlFormAssociationState::EstablishedCurrent(Some(f.ids[0].clone())),
            ),
        ],
    );
    let changed = report(&f, &c, HtmlSemanticQuery::FormDefaultButton);
    assert!(
        matches!(default(&changed),HtmlSemanticValue::Known(Some(v)) if v.subject()==&f.ids[2])
    );
    assert_ne!(
        r.qualification().context(),
        changed.qualification().context()
    );
    assert_eq!(r.capture(), changed.capture());
}

#[test]
fn external_button_before_form_precedes_its_descendant_button() {
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", &[]);
    let external = element(&mut b, &d, "input", &[("type", "submit")]);
    let descendant = element(&mut b, &d, "button", &[]);
    b.establish_children(&external, &[]).unwrap();
    b.establish_children(&form, std::slice::from_ref(&descendant))
        .unwrap();
    b.establish_children(&d, &[external.clone(), form.clone()])
        .unwrap();
    let f = finish(b, &[form, external, descendant]);
    let c = owned(&f, &[1, 2]);
    assert!(
        matches!(default(&report(&f, &c, HtmlSemanticQuery::FormDefaultButton)), HtmlSemanticValue::Known(Some(v)) if v.subject() == &f.ids[1])
    );
}

#[test]
fn command_presence_excludes_auto_without_reading_large_command_values() {
    let huge = "x".repeat(1000);
    for name in ["command", "commandfor"] {
        let f = flat(&[("button", &[(name, &huge)])]);
        let c = owned(&f, &[]);
        let mut s = session(&f, &c, limits(), 0, HtmlSemanticCancellation::new());
        let r = s
            .query(&[HtmlSemanticRequest::new(
                f.ids[0].clone(),
                HtmlSemanticQuery::FormImplicitSubmissionCondition,
            )])
            .unwrap();
        assert_eq!(blockers(&r), HtmlImplicitSubmissionBlockingFields::Zero);
    }
}
#[test]
fn complete_empty_tree_proves_no_button_and_zero_fields() {
    let f = flat(&[]);
    let c = owned(&f, &[]);
    assert_eq!(
        default(&report(&f, &c, HtmlSemanticQuery::FormDefaultButton)),
        &HtmlSemanticValue::Known(None)
    );
    assert_eq!(
        blockers(&report(
            &f,
            &c,
            HtmlSemanticQuery::FormImplicitSubmissionCondition
        )),
        HtmlImplicitSubmissionBlockingFields::Zero
    );
}
#[test]
fn null_owner_is_not_missing_and_missing_earlier_owner_prevents_later_first() {
    let f = flat(&[
        ("button", &[("type", "submit")]),
        ("input", &[("type", "submit")]),
    ]);
    let c = context(
        &f,
        vec![
            (1, HtmlFormAssociationState::EstablishedCurrent(None)),
            (
                2,
                HtmlFormAssociationState::EstablishedCurrent(Some(f.ids[0].clone())),
            ),
        ],
    );
    assert!(
        matches!(default(&report(&f,&c,HtmlSemanticQuery::FormDefaultButton)),HtmlSemanticValue::Known(Some(v)) if v.subject()==&f.ids[2])
    );
    let c = owned(&f, &[2]);
    assert_eq!(
        default(&report(&f, &c, HtmlSemanticQuery::FormDefaultButton)),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::FormAssociation(f.ids[1].clone()))
    );
}
#[test]
fn unknown_type_auto_commands_and_select_parent_do_not_become_submit_buttons() {
    for attrs in [
        &[("type", "reset")][..],
        &[("type", "button")][..],
        &[("command", "x")][..],
        &[("commandfor", "x")][..],
    ] {
        let f = flat(&[("button", attrs)]);
        let c = owned(&f, &[]);
        assert_eq!(
            default(&report(&f, &c, HtmlSemanticQuery::FormDefaultButton)),
            &HtmlSemanticValue::Known(None)
        );
    }
    for attrs in [&[][..], &[("type", "bad")][..]] {
        let f = flat(&[("button", attrs)]);
        let c = owned(&f, &[1]);
        assert!(
            matches!(default(&report(&f,&c,HtmlSemanticQuery::FormDefaultButton)),HtmlSemanticValue::Known(Some(v)) if v.subject()==&f.ids[1])
        );
    }
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", &[]);
    let select = element(&mut b, &d, "select", &[]);
    let button = element(&mut b, &d, "button", &[]);
    b.establish_children(&button, &[]).unwrap();
    b.establish_children(&select, std::slice::from_ref(&button))
        .unwrap();
    b.establish_children(&form, &[]).unwrap();
    b.establish_children(&d, &[select.clone(), form.clone()])
        .unwrap();
    let f = finish(b, &[form, button]);
    let c = owned(&f, &[]);
    assert_eq!(
        default(&report(&f, &c, HtmlSemanticQuery::FormDefaultButton)),
        &HtmlSemanticValue::Known(None)
    );
}
#[test]
fn earlier_unavailable_subtree_blocks_selection_but_trailing_subtree_is_irrelevant() {
    for before in [true, false] {
        let (mut b, d) = builder();
        let form = element(&mut b, &d, "form", &[]);
        let branch = element(&mut b, &d, "div", &[]);
        let button = element(&mut b, &d, "button", &[("type", "submit")]);
        b.establish_children(&form, &[]).unwrap();
        let children = if before {
            vec![branch.clone(), button.clone(), form.clone()]
        } else {
            vec![button.clone(), branch.clone(), form.clone()]
        };
        b.establish_children(&d, &children).unwrap();
        let f = finish(b, &[form, button, branch]);
        let c = owned(&f, &[1]);
        let r = report(&f, &c, HtmlSemanticQuery::FormDefaultButton);
        if before {
            assert_eq!(
                default(&r),
                &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(f.ids[2].clone()))
            );
        } else {
            assert!(
                matches!(default(&r),HtmlSemanticValue::Known(Some(v)) if v.subject()==&f.ids[1])
            );
        }
    }
}
#[test]
fn unavailable_form_parent_and_children_are_distinct_from_empty() {
    for root in [false, true] {
        let (mut b, d) = builder();
        let form = element(&mut b, &d, "form", &[]);
        if root {
            b.establish_root(&form).unwrap();
        }
        let f = finish(b, &[form]);
        let c = owned(&f, &[]);
        let expected = if root {
            HtmlSemanticMissing::OrderedChildren(f.ids[0].clone())
        } else {
            HtmlSemanticMissing::Parent(f.ids[0].clone())
        };
        assert_eq!(
            default(&report(&f, &c, HtmlSemanticQuery::FormDefaultButton)),
            &HtmlSemanticValue::Missing(expected)
        );
    }
}
#[test]
fn detached_complete_form_tree_is_evaluable() {
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", &[]);
    let button = element(&mut b, &d, "button", &[]);
    b.establish_root(&form).unwrap();
    b.establish_children(&form, std::slice::from_ref(&button))
        .unwrap();
    let f = finish(b, &[form, button]);
    let c = context(&f, vec![(1, HtmlFormAssociationState::ResetAgainstCapture)]);
    let r = report(&f, &c, HtmlSemanticQuery::FormDefaultButton);
    assert!(
        matches!(default(&r),HtmlSemanticValue::Known(Some(v)) if v.subject()==&f.ids[1] && v.form_owner().origin()==HtmlFormOwnerOrigin::ResetAgainstCapture)
    );
}
#[test]
fn reset_external_association_uses_first_equal_id_element_not_first_form() {
    for obstruct in [false, true] {
        let (mut b, d) = builder();
        let form = element(&mut b, &d, "form", &[("id", "f")]);
        let other = element(&mut b, &d, "div", &[("id", "f")]);
        let button = element(&mut b, &d, "input", &[("type", "image"), ("form", "f")]);
        for h in [&form, &other, &button] {
            b.establish_children(h, &[]).unwrap();
        }
        let order = if obstruct {
            vec![other, button.clone(), form.clone()]
        } else {
            vec![button.clone(), form.clone(), other]
        };
        b.establish_children(&d, &order).unwrap();
        let f = finish(b, &[form, button]);
        let c = context(&f, vec![(1, HtmlFormAssociationState::ResetAgainstCapture)]);
        let r = report(&f, &c, HtmlSemanticQuery::FormDefaultButton);
        if obstruct {
            assert_eq!(default(&r), &HtmlSemanticValue::Known(None));
        } else {
            assert!(
                matches!(default(&r),HtmlSemanticValue::Known(Some(v)) if v.subject()==&f.ids[1] && v.form_owner().origin()==HtmlFormOwnerOrigin::ResetAgainstCapture)
            );
        }
    }
}
#[test]
fn each_source_blocking_input_state_counts_even_disabled_and_readonly() {
    for state in [
        "text",
        "search",
        "tel",
        "url",
        "email",
        "password",
        "date",
        "month",
        "week",
        "time",
        "datetime-local",
        "number",
        "",
        "invalid",
    ] {
        let f = flat(&[(
            "input",
            &[("type", state), ("disabled", "false"), ("readonly", "")],
        )]);
        let c = owned(&f, &[1]);
        assert_eq!(
            blockers(&report(
                &f,
                &c,
                HtmlSemanticQuery::FormImplicitSubmissionCondition
            )),
            HtmlImplicitSubmissionBlockingFields::One,
            "{state}"
        );
    }
    let f = flat(&[("input", &[])]);
    let c = owned(&f, &[1]);
    assert_eq!(
        blockers(&report(
            &f,
            &c,
            HtmlSemanticQuery::FormImplicitSubmissionCondition
        )),
        HtmlImplicitSubmissionBlockingFields::One
    );
}
#[test]
fn nonblocking_states_and_textarea_need_no_owner_for_field_count() {
    for state in [
        "hidden", "range", "color", "checkbox", "radio", "file", "button", "reset",
    ] {
        let f = flat(&[("input", &[("type", state)])]);
        let c = owned(&f, &[]);
        assert_eq!(
            blockers(&report(
                &f,
                &c,
                HtmlSemanticQuery::FormImplicitSubmissionCondition
            )),
            HtmlImplicitSubmissionBlockingFields::Zero,
            "{state}"
        );
    }
    let f = flat(&[("textarea", &[])]);
    let c = owned(&f, &[]);
    assert_eq!(
        blockers(&report(
            &f,
            &c,
            HtmlSemanticQuery::FormImplicitSubmissionCondition
        )),
        HtmlImplicitSubmissionBlockingFields::Zero
    );
}
#[test]
fn two_known_fields_prove_multiple_despite_other_unknown_owner() {
    let f = flat(&[("input", &[]), ("input", &[]), ("input", &[])]);
    for known in [&[1, 2][..], &[2, 3][..]] {
        let c = owned(&f, known);
        assert_eq!(
            blockers(&report(
                &f,
                &c,
                HtmlSemanticQuery::FormImplicitSubmissionCondition
            )),
            HtmlImplicitSubmissionBlockingFields::Multiple
        );
    }
    let c = owned(&f, &[2]);
    assert_eq!(
        condition(&report(
            &f,
            &c,
            HtmlSemanticQuery::FormImplicitSubmissionCondition
        )),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::FormAssociation(f.ids[1].clone()))
    );
    let c = context(
        &f,
        vec![
            (1, HtmlFormAssociationState::EstablishedCurrent(None)),
            (
                2,
                HtmlFormAssociationState::EstablishedCurrent(Some(f.ids[0].clone())),
            ),
            (3, HtmlFormAssociationState::EstablishedCurrent(None)),
        ],
    );
    assert_eq!(
        blockers(&report(
            &f,
            &c,
            HtmlSemanticQuery::FormImplicitSubmissionCondition
        )),
        HtmlImplicitSubmissionBlockingFields::One
    );
}
#[test]
fn default_branch_skips_unavailable_blocker_owners() {
    let f = flat(&[("input", &[]), ("input", &[("type", "submit")])]);
    let c = owned(&f, &[2]);
    assert!(
        matches!(condition(&report(&f,&c,HtmlSemanticQuery::FormImplicitSubmissionCondition)),HtmlSemanticValue::Known(HtmlImplicitSubmissionCondition::DefaultButton(v)) if v.subject()==&f.ids[2])
    );
}
#[test]
fn exact_work_output_scratch_bytes_and_raw_limits_are_atomic_and_sticky() {
    let f = flat(&[("button", &[("type", "submit")])]);
    let c = owned(&f, &[1]);
    let req = HtmlSemanticRequest::new(f.ids[0].clone(), HtmlSemanticQuery::FormDefaultButton);
    let baseline = report(&f, &c, HtmlSemanticQuery::FormDefaultButton).usage();
    assert_eq!(baseline.nested_output_records(), 0);
    assert!(baseline.peak_scratch_records() > 0);
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
                    1000
                },
                0,
                if resource == HtmlSemanticResource::AllocatedBytes {
                    cap
                } else {
                    1000000
                },
                0,
            );
            let mut s = session(&f, &c, l, 6, HtmlSemanticCancellation::new());
            let result = s.query(std::slice::from_ref(&req));
            if cap == exact {
                assert!(result.is_ok(), "{resource:?}: {result:?}");
            } else {
                let e = result.unwrap_err();
                assert!(
                    matches!(e.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r==resource),
                    "{e:?}"
                );
                assert_eq!(e.qualification().unwrap().limits(), l);
                assert_eq!(s.query(&[]).unwrap_err(), e);
            }
            assert_eq!(s.usage().scratch_records(), 0);
        }
    }
    let mut s = session(&f, &c, limits(), 5, HtmlSemanticCancellation::new());
    let err = s
        .query(&[
            HtmlSemanticRequest::new(f.ids[0].clone(), HtmlSemanticQuery::Hidden),
            req,
        ])
        .unwrap_err();
    assert!(matches!(err.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(s.query(&[]).unwrap_err(), err);
    assert_eq!(s.usage().scratch_records(), 0);
}
#[test]
fn cancelled_and_foreign_capture_failures_retain_original_qualification() {
    let f = flat(&[]);
    let c = owned(&f, &[]);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut s = session(&f, &c, limits(), 10, cancel);
    let e = s
        .query(&[HtmlSemanticRequest::new(
            f.ids[0].clone(),
            HtmlSemanticQuery::FormDefaultButton,
        )])
        .unwrap_err();
    assert_eq!(e.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(e.qualification().unwrap().context(), c.identity());
    assert_eq!(s.query(&[]).unwrap_err(), e);
    let other = flat(&[]);
    let mut s = session(&f, &c, limits(), 10, HtmlSemanticCancellation::new());
    let e = s
        .query(&[HtmlSemanticRequest::new(
            other.ids[0].clone(),
            HtmlSemanticQuery::FormDefaultButton,
        )])
        .unwrap_err();
    assert!(matches!(e.kind(), HtmlSemanticFailureKind::Capture(_)));
    assert_eq!(e.qualification().unwrap().capture(), f.capture.identity());
    assert_eq!(s.query(&[]).unwrap_err(), e);
    assert!(matches!(
        HtmlSemanticSession::new(
            &other.capture,
            &c,
            HtmlValueInterpreter::new(10),
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
fn foreign_candidates_are_excluded_and_wrong_query_roles_remain_distinct() {
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", &[]);
    let foreign = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::svg(), None, "button"),
            attributes(&[("type", "submit")]),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let wrong = element(&mut b, &d, "FORM", &[]);
    for h in [&form, &foreign, &wrong] {
        b.establish_children(h, &[]).unwrap();
    }
    b.establish_children(&d, &[foreign.clone(), form.clone(), wrong.clone()])
        .unwrap();
    let f = finish(b, &[form, foreign, wrong, d]);
    let c = owned(&f, &[]);
    assert_eq!(
        default(&report(&f, &c, HtmlSemanticQuery::FormDefaultButton)),
        &HtmlSemanticValue::Known(None)
    );
    for query in [
        HtmlSemanticQuery::FormDefaultButton,
        HtmlSemanticQuery::FormImplicitSubmissionCondition,
    ] {
        let mut s = session(&f, &c, limits(), 0, HtmlSemanticCancellation::new());
        let r = s
            .query(&[
                HtmlSemanticRequest::new(f.ids[1].clone(), query),
                HtmlSemanticRequest::new(f.ids[2].clone(), query),
                HtmlSemanticRequest::new(f.ids[3].clone(), query),
            ])
            .unwrap();
        for (i, res) in r.results().iter().enumerate() {
            match res.fact() {
                HtmlSemanticFact::FormDefaultButton(v) => assert_eq!(
                    v,
                    &if i == 0 {
                        HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
                    } else {
                        HtmlSemanticValue::Inapplicable
                    }
                ),
                HtmlSemanticFact::FormImplicitSubmissionCondition(v) => assert_eq!(
                    v,
                    &if i == 0 {
                        HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
                    } else {
                        HtmlSemanticValue::Inapplicable
                    }
                ),
                other => panic!("{other:?}"),
            }
        }
    }
}
#[test]
fn template_and_shadow_hosts_are_not_ordinary_traversal_edges() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let host = element(&mut b, &d, if shadow { "div" } else { "template" }, &[]);
        let fragment = if shadow {
            b.add_shadow_root(
                &d,
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
        let form = element(&mut b, &d, "form", &[]);
        let outside = element(&mut b, &d, "button", &[("type", "submit")]);
        b.establish_children(&form, &[]).unwrap();
        b.establish_children(&fragment, std::slice::from_ref(&form))
            .unwrap();
        b.establish_children(&host, std::slice::from_ref(&outside))
            .unwrap();
        b.establish_children(&d, std::slice::from_ref(&host))
            .unwrap();
        let f = finish(b, &[form, outside]);
        let c = owned(&f, &[]);
        assert_eq!(
            default(&report(&f, &c, HtmlSemanticQuery::FormDefaultButton)),
            &HtmlSemanticValue::Known(None)
        );
        assert_eq!(
            blockers(&report(
                &f,
                &c,
                HtmlSemanticQuery::FormImplicitSubmissionCondition
            )),
            HtmlImplicitSubmissionBlockingFields::Zero
        );
    }
}
#[test]
fn fallback_scan_has_exact_budget_and_duplicate_batch_order() {
    let f = flat(&[("input", &[]), ("input", &[])]);
    let c = owned(&f, &[1, 2]);
    let req = HtmlSemanticRequest::new(
        f.ids[0].clone(),
        HtmlSemanticQuery::FormImplicitSubmissionCondition,
    );
    let baseline = report(&f, &c, HtmlSemanticQuery::FormImplicitSubmissionCondition).usage();
    for cap in [baseline.work(), baseline.work() - 1] {
        let l = HtmlSemanticLimits::new(
            cap,
            1,
            baseline.peak_scratch_records(),
            0,
            baseline.allocated_bytes(),
            0,
        );
        let mut s = session(&f, &c, l, 0, HtmlSemanticCancellation::new());
        let r = s.query(std::slice::from_ref(&req));
        if cap == baseline.work() {
            assert_eq!(
                blockers(&r.unwrap()),
                HtmlImplicitSubmissionBlockingFields::Multiple
            );
        } else {
            let e = r.unwrap_err();
            assert!(matches!(
                e.kind(),
                HtmlSemanticFailureKind::LimitExceeded {
                    resource: HtmlSemanticResource::Work,
                    ..
                }
            ));
            assert_eq!(s.query(&[]).unwrap_err(), e);
        }
        assert_eq!(s.usage().scratch_records(), 0);
    }
    let mut s = session(&f, &c, limits(), 0, HtmlSemanticCancellation::new());
    let r = s.query(&[req.clone(), req]).unwrap();
    assert_eq!(r.results().len(), 2);
    assert_eq!(r.results()[0], r.results()[1]);
    assert_eq!(
        r.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
    let mut s = session(
        &f,
        &c,
        HtmlSemanticLimits::new(10000000, 1, 1000, 0, 1000000, 0),
        0,
        HtmlSemanticCancellation::new(),
    );
    assert!(matches!(
        s.query(&[
            HtmlSemanticRequest::new(f.ids[0].clone(), HtmlSemanticQuery::FormDefaultButton),
            HtmlSemanticRequest::new(
                f.ids[0].clone(),
                HtmlSemanticQuery::FormImplicitSubmissionCondition
            )
        ])
        .unwrap_err()
        .kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::OutputRecords,
            ..
        }
    ));
}
