use crate::semantics::support::{
    moderate_semantic_limits as limits, small_builder_without_registries as builder,
};
use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlControlFacts, HtmlControlFactsLimits,
    HtmlFormAssociation, HtmlFormAssociationState, HtmlFormOwnerOrigin, HtmlNoValidateResolution,
    HtmlSemanticCancellation, HtmlSemanticContext, HtmlSemanticContextInput, HtmlSemanticFact,
    HtmlSemanticFailureKind, HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery,
    HtmlSemanticReport, HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticSession,
    HtmlSemanticUnsupported, HtmlSemanticValue, HtmlSubmitterNoValidate,
};

fn attributes(values: &[(&str, &str)]) -> OrderedAttributes {
    OrderedAttributes::new(values.iter().map(|(name, value)| {
        Attribute::new(QualifiedName::new(Namespace::none(), None, *name), *value)
    }))
    .unwrap()
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
    control: HtmlCaptureSubjectId,
    form: HtmlCaptureSubjectId,
}
fn fixture(
    local: &str,
    attrs: &[(&str, &str)],
    form_attrs: &[(&str, &str)],
    linked: bool,
) -> Fixture {
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", form_attrs);
    let control = element(&mut b, &d, local, attrs);
    if linked {
        b.establish_children(&form, std::slice::from_ref(&control))
            .unwrap();
        b.establish_children(&d, std::slice::from_ref(&form))
            .unwrap();
    }
    let capture = b.finish().unwrap();
    Fixture {
        control: capture.subject_id(&control).unwrap(),
        form: capture.subject_id(&form).unwrap(),
        capture,
    }
}
fn context(f: &Fixture, state: Option<HtmlFormAssociationState>) -> HtmlSemanticContext {
    let mut input = HtmlSemanticContextInput::new(vec![]);
    if let Some(state) = state {
        input = input.with_controls(
            HtmlControlFacts::new(
                &f.capture,
                f.capture.host_facts_revision().clone(),
                vec![HtmlFormAssociation::new(f.control.clone(), state)],
                vec![],
                vec![],
                vec![],
                HtmlControlFactsLimits::new(10, 1000000),
            )
            .unwrap(),
        );
    }
    HtmlSemanticContext::new(
        &f.capture,
        f.capture.host_facts_revision().clone(),
        input,
        0,
    )
    .unwrap()
}
fn owner_context(f: &Fixture) -> HtmlSemanticContext {
    context(
        f,
        Some(HtmlFormAssociationState::EstablishedCurrent(Some(
            f.form.clone(),
        ))),
    )
}
fn session<'a>(
    f: &'a Fixture,
    cx: &'a HtmlSemanticContext,
    l: HtmlSemanticLimits,
    raw: usize,
    cancel: HtmlSemanticCancellation,
) -> HtmlSemanticSession<'a> {
    HtmlSemanticSession::new(&f.capture, cx, HtmlValueInterpreter::new(raw), l, cancel).unwrap()
}
fn request(f: &Fixture, query: HtmlSemanticQuery) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(f.control.clone(), query)
}
fn report(f: &Fixture, cx: &HtmlSemanticContext) -> HtmlSemanticReport {
    session(f, cx, limits(), 100000, HtmlSemanticCancellation::new())
        .query(&[request(f, HtmlSemanticQuery::SubmitterNoValidate)])
        .unwrap()
}
fn value(r: &HtmlSemanticReport) -> &HtmlSubmitterNoValidate {
    match r.results()[0].fact() {
        HtmlSemanticFact::SubmitterNoValidate(HtmlSemanticValue::Known(v)) => v,
        other => panic!("expected no-validate observation: {other:?}"),
    }
}

#[test]
fn own_submit_presence_proves_true_without_owner_or_ancestry() {
    for local in ["input", "button"] {
        for state in if local == "input" {
            &["submit", "image"][..]
        } else {
            &["submit"][..]
        } {
            for raw in ["", "formnovalidate", "FORMNOVALIDATE", "false"] {
                let f = fixture(
                    local,
                    &[("type", state), ("formnovalidate", raw)],
                    &[],
                    false,
                );
                let r = report(&f, &context(&f, None));
                let v = value(&r);
                assert_eq!(v.value(), Some(true));
                assert_eq!(v.resolution(), &HtmlNoValidateResolution::OwnSubmitter);
                assert_eq!(v.intrinsic().intrinsic(), Some(&true));
                assert_eq!(v.intrinsic().raw(), Some(&DocumentString::from(raw)));
                assert_eq!(
                    v.intrinsic().context_author(),
                    HtmlAttributeContextAuthorState::Conforming
                );
                if raw == "false" {
                    assert_eq!(
                        v.intrinsic().author(),
                        HtmlAttributeAuthorState::Nonconforming(
                            HtmlAttributeConformanceError::BooleanSpelling
                        )
                    );
                } else {
                    assert_eq!(v.intrinsic().author(), HtmlAttributeAuthorState::Conforming);
                }
                assert!(v.form_attribute().is_none());
            }
        }
    }
}

#[test]
fn absent_own_attribute_skips_type_and_unknown_parent_and_uses_owner() {
    let huge = "x".repeat(1000);
    for form in [vec![], vec![("novalidate", "")]] {
        let f = fixture("button", &[("type", &huge)], &form, false);
        let cx = owner_context(&f);
        let r = session(&f, &cx, limits(), 0, HtmlSemanticCancellation::new())
            .query(&[request(&f, HtmlSemanticQuery::SubmitterNoValidate)])
            .unwrap();
        let v = value(&r);
        assert_eq!(v.value(), Some(!form.is_empty()));
        assert_eq!(v.intrinsic().intrinsic(), Some(&false));
        assert_eq!(v.intrinsic().raw(), None);
        assert!(v.submitter_type().is_none());
        assert!(
            matches!(v.resolution(), HtmlNoValidateResolution::FormOwner{value,owner}
            if *value==!form.is_empty() && owner.owner()==Some(&f.form))
        );
    }
}

#[test]
fn non_submit_inputs_keep_true_content_and_forbidden_authorship_but_use_owner() {
    for state in [
        None,
        Some("unknown"),
        Some("text"),
        Some("file"),
        Some("reset"),
        Some("button"),
    ] {
        for present in [false, true] {
            let mut attrs = vec![("formnovalidate", "false")];
            if let Some(state) = state {
                attrs.push(("type", state));
            }
            let forms = if present {
                vec![("novalidate", "")]
            } else {
                vec![]
            };
            let f = fixture("input", &attrs, &forms, false);
            let r = report(&f, &owner_context(&f));
            let v = value(&r);
            assert_eq!(v.value(), Some(present));
            assert_eq!(v.intrinsic().intrinsic(), Some(&true));
            assert_eq!(
                v.intrinsic().context_author(),
                HtmlAttributeContextAuthorState::Nonconforming(
                    HtmlAttributeContextConformanceError::InputStateForbidsAttribute
                )
            );
            assert_eq!(
                v.intrinsic().author(),
                HtmlAttributeAuthorState::Nonconforming(
                    HtmlAttributeConformanceError::BooleanSpelling
                )
            );
        }
    }
}

#[test]
fn non_submit_buttons_keep_author_violation_and_use_owner_without_parent() {
    for state in ["reset", "button"] {
        let f = fixture(
            "button",
            &[("type", state), ("formnovalidate", "")],
            &[],
            false,
        );
        let r = report(&f, &owner_context(&f));
        let v = value(&r);
        assert_eq!(v.value(), Some(false));
        assert_eq!(v.intrinsic().intrinsic(), Some(&true));
        assert_eq!(
            v.intrinsic().button_submitter_context(),
            Some(HtmlButtonSubmitterContext::NotSubmitButton)
        );
        assert_eq!(
            v.intrinsic().context_author(),
            HtmlAttributeContextAuthorState::Nonconforming(
                HtmlAttributeContextConformanceError::FormnovalidateRequiresSubmitButton
            )
        );
    }
}

#[test]
fn unknown_auto_eligibility_does_not_hide_a_true_owner_disjunct() {
    for raw in ["", "novalidate", "false"] {
        let f = fixture(
            "button",
            &[("formnovalidate", "")],
            &[("novalidate", raw)],
            false,
        );
        let r = report(&f, &owner_context(&f));
        let v = value(&r);
        assert_eq!(v.value(), Some(true));
        assert!(
            matches!(v.resolution(), HtmlNoValidateResolution::FormOwner{value:true,owner}
            if owner.owner()==Some(&f.form))
        );
        assert_eq!(
            v.intrinsic().context_author(),
            HtmlAttributeContextAuthorState::Unchecked
        );
        assert_eq!(v.intrinsic().button_submitter_context(), None);
        assert_eq!(v.form_attribute().unwrap().intrinsic(), Some(&true));
        assert_eq!(
            v.form_attribute().unwrap().raw(),
            Some(&DocumentString::from(raw))
        );
    }
}

#[test]
fn unknown_auto_with_false_owner_remains_missing_and_retains_known_owner() {
    let f = fixture("button", &[("formnovalidate", "")], &[], false);
    for is_null in [false, true] {
        let cx = if is_null {
            context(&f, Some(HtmlFormAssociationState::EstablishedCurrent(None)))
        } else {
            owner_context(&f)
        };
        let r = report(&f, &cx);
        let v = value(&r);
        assert_eq!(v.value(), None);
        assert!(
            matches!(v.resolution(), HtmlNoValidateResolution::Missing { missing, owner:Some(owner) }
            if missing==&HtmlSemanticMissing::Parent(f.control.clone()) && owner.owner().is_none()==is_null
                && owner.origin()==HtmlFormOwnerOrigin::EstablishedCurrent)
        );
        assert_eq!(v.form_attribute().is_none(), is_null);
        if !is_null {
            assert_eq!(v.form_attribute().unwrap().intrinsic(), Some(&false));
        }
    }
}

#[test]
fn absent_own_and_actual_null_prove_false_while_unknown_association_does_not() {
    let f = fixture("input", &[], &[], true);
    let r = report(
        &f,
        &context(&f, Some(HtmlFormAssociationState::EstablishedCurrent(None))),
    );
    let v = value(&r);
    assert_eq!(v.value(), Some(false));
    assert!(
        matches!(v.resolution(), HtmlNoValidateResolution::NoFormOwner(owner)
        if owner.owner().is_none() && owner.origin()==HtmlFormOwnerOrigin::EstablishedCurrent)
    );
    assert!(v.form_attribute().is_none());
    for state in [None, Some(HtmlFormAssociationState::Unavailable)] {
        let r = report(&f, &context(&f, state));
        assert_eq!(value(&r).value(), None);
        assert_eq!(
            value(&r).resolution(),
            &HtmlNoValidateResolution::Missing {
                missing: HtmlSemanticMissing::FormAssociation(f.control.clone()),
                owner: None,
            }
        );
    }
}

#[test]
fn known_non_submit_and_missing_owner_do_not_guess_false_from_own_presence() {
    let f = fixture(
        "input",
        &[("type", "text"), ("formnovalidate", "")],
        &[],
        true,
    );
    let r = report(&f, &context(&f, None));
    assert_eq!(value(&r).value(), None);
    assert_eq!(
        value(&r).resolution(),
        &HtmlNoValidateResolution::Missing {
            missing: HtmlSemanticMissing::FormAssociation(f.control.clone()),
            owner: None,
        }
    );
    assert_eq!(value(&r).intrinsic().intrinsic(), Some(&true));
}

#[test]
fn auto_select_buttons_use_owner_independently_of_child_position() {
    for first in [false, true] {
        let (mut b, d) = builder();
        let form = element(&mut b, &d, "form", &[]);
        let select = element(&mut b, &d, "select", &[]);
        let other = element(&mut b, &d, "span", &[]);
        let control = element(
            &mut b,
            &d,
            "button",
            &[("type", "invalid"), ("formnovalidate", "")],
        );
        let children = if first {
            vec![control.clone(), other]
        } else {
            vec![other, control.clone()]
        };
        b.establish_children(&select, &children).unwrap();
        let capture = b.finish().unwrap();
        let f = Fixture {
            control: capture.subject_id(&control).unwrap(),
            form: capture.subject_id(&form).unwrap(),
            capture,
        };
        let r = report(&f, &owner_context(&f));
        let v = value(&r);
        assert_eq!(v.value(), Some(false));
        assert_eq!(
            v.intrinsic().button_submitter_context(),
            Some(HtmlButtonSubmitterContext::NotSubmitButton)
        );
        assert_eq!(
            v.intrinsic().context_author(),
            HtmlAttributeContextAuthorState::Nonconforming(
                HtmlAttributeContextConformanceError::FormnovalidateRequiresSubmitButton
            )
        );
    }
}

#[test]
fn auto_command_presence_uses_owner_without_command_value_or_parent_reads() {
    let huge = "x".repeat(1000);
    for name in ["command", "commandfor"] {
        let f = fixture(
            "button",
            &[(name, &huge), ("formnovalidate", "")],
            &[("novalidate", "")],
            false,
        );
        let cx = owner_context(&f);
        let r = session(&f, &cx, limits(), 0, HtmlSemanticCancellation::new())
            .query(&[request(&f, HtmlSemanticQuery::SubmitterNoValidate)])
            .unwrap();
        let v = value(&r);
        assert_eq!(v.value(), Some(true));
        assert_eq!(
            v.intrinsic().button_submitter_context(),
            Some(HtmlButtonSubmitterContext::NotSubmitButton)
        );
        assert!(matches!(
            v.resolution(),
            HtmlNoValidateResolution::FormOwner { value: true, .. }
        ));
    }
}

#[test]
fn auto_root_and_non_select_parent_prove_own_true_without_association() {
    for root in [false, true] {
        let (mut b, d) = builder();
        let form = element(&mut b, &d, "form", &[]);
        let parent = element(&mut b, &d, "div", &[]);
        let control = element(&mut b, &d, "button", &[("formnovalidate", "")]);
        if root {
            b.establish_root(&control).unwrap();
        } else {
            b.establish_parent(&control, &parent).unwrap();
        }
        let capture = b.finish().unwrap();
        let f = Fixture {
            control: capture.subject_id(&control).unwrap(),
            form: capture.subject_id(&form).unwrap(),
            capture,
        };
        let r = report(&f, &context(&f, None));
        assert_eq!(
            value(&r).resolution(),
            &HtmlNoValidateResolution::OwnSubmitter
        );
        assert_eq!(value(&r).value(), Some(true));
    }
}

#[test]
fn explicit_submit_proof_ignores_disabled_select_parent_and_large_owner_raw() {
    let (mut b, d) = builder();
    let huge = "x".repeat(1000);
    let form = element(&mut b, &d, "form", &[("novalidate", &huge)]);
    let select = element(&mut b, &d, "select", &[]);
    let control = element(
        &mut b,
        &d,
        "button",
        &[
            ("type", "submit"),
            ("formnovalidate", ""),
            ("command", &huge),
            ("disabled", &huge),
        ],
    );
    b.establish_parent(&control, &select).unwrap();
    let capture = b.finish().unwrap();
    let f = Fixture {
        control: capture.subject_id(&control).unwrap(),
        form: capture.subject_id(&form).unwrap(),
        capture,
    };
    let cx = owner_context(&f);
    let r = session(&f, &cx, limits(), 6, HtmlSemanticCancellation::new())
        .query(&[request(&f, HtmlSemanticQuery::SubmitterNoValidate)])
        .unwrap();
    assert_eq!(
        value(&r).resolution(),
        &HtmlNoValidateResolution::OwnSubmitter
    );
    assert_eq!(value(&r).value(), Some(true));
    assert!(value(&r).form_attribute().is_none());
}

#[test]
fn reset_owner_and_root_null_keep_source_association_origins() {
    let f = fixture("input", &[], &[("novalidate", "false")], true);
    let cx = context(&f, Some(HtmlFormAssociationState::ResetAgainstCapture));
    let r = report(&f, &cx);
    assert_eq!(value(&r).value(), Some(true));
    assert!(
        matches!(value(&r).resolution(), HtmlNoValidateResolution::FormOwner{value:true,owner}
        if owner.owner()==Some(&f.form) && owner.origin()==HtmlFormOwnerOrigin::ResetAgainstCapture)
    );
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", &[]);
    let control = element(&mut b, &d, "input", &[]);
    b.establish_root(&control).unwrap();
    let capture = b.finish().unwrap();
    let f = Fixture {
        control: capture.subject_id(&control).unwrap(),
        form: capture.subject_id(&form).unwrap(),
        capture,
    };
    let cx = context(&f, Some(HtmlFormAssociationState::ResetAgainstCapture));
    let r = report(&f, &cx);
    assert_eq!(value(&r).value(), Some(false));
    assert!(
        matches!(value(&r).resolution(), HtmlNoValidateResolution::NoFormOwner(owner)
        if owner.owner().is_none() && owner.origin()==HtmlFormOwnerOrigin::ResetAgainstCapture)
    );
}

#[test]
fn surrogate_boolean_content_preserves_exact_raw_for_both_disjuncts() {
    for own in [false, true] {
        let (mut b, d) = builder();
        let raw = DocumentString::from_utf16(vec![0xd800, 0x20, 0x66, 0x61, 0x6c, 0x73, 0x65]);
        let records = OrderedAttributes::new([Attribute::new(
            QualifiedName::new(
                Namespace::none(),
                Some("kept".into()),
                if own { "formnovalidate" } else { "novalidate" },
            ),
            raw.clone(),
        )])
        .unwrap();
        let form = if own {
            element(&mut b, &d, "form", &[])
        } else {
            b.add_element(
                &d,
                QualifiedName::new(Namespace::html(), None, "form"),
                records.clone(),
                HtmlCapturedIsValue::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap()
        };
        let control = if own {
            let mut attrs = records.iter().cloned().collect::<Vec<_>>();
            attrs.push(Attribute::new(
                QualifiedName::new(Namespace::none(), None, "type"),
                "submit",
            ));
            b.add_element(
                &d,
                QualifiedName::new(Namespace::html(), None, "input"),
                OrderedAttributes::new(attrs).unwrap(),
                HtmlCapturedIsValue::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap()
        } else {
            element(&mut b, &d, "input", &[])
        };
        let capture = b.finish().unwrap();
        let f = Fixture {
            control: capture.subject_id(&control).unwrap(),
            form: capture.subject_id(&form).unwrap(),
            capture,
        };
        let r = report(&f, &owner_context(&f));
        let v = value(&r);
        assert_eq!(v.value(), Some(true));
        let intrinsic = if own {
            v.intrinsic()
        } else {
            v.form_attribute().unwrap()
        };
        assert_eq!(intrinsic.raw(), Some(&raw));
        assert_eq!(intrinsic.intrinsic(), Some(&true));
        assert_eq!(
            intrinsic.author(),
            HtmlAttributeAuthorState::Nonconforming(HtmlAttributeConformanceError::BooleanSpelling)
        );
        assert_eq!(
            intrinsic.raw_attribute().unwrap().name().prefix(),
            Some(&DocumentString::from("kept"))
        );
    }
}

#[test]
fn current_nonancestor_owner_changes_qualified_result_without_ancestor_substitution() {
    let (mut b, d) = builder();
    let ancestor = element(&mut b, &d, "form", &[]);
    let actual = element(&mut b, &d, "form", &[("novalidate", "")]);
    let control = element(&mut b, &d, "input", &[]);
    b.establish_children(&ancestor, std::slice::from_ref(&control))
        .unwrap();
    b.establish_children(&actual, &[]).unwrap();
    b.establish_children(&d, &[ancestor.clone(), actual.clone()])
        .unwrap();
    let capture = b.finish().unwrap();
    let f = Fixture {
        control: capture.subject_id(&control).unwrap(),
        form: capture.subject_id(&actual).unwrap(),
        capture,
    };
    let cx = owner_context(&f);
    let r = report(&f, &cx);
    assert_eq!(value(&r).value(), Some(true));
    assert!(
        matches!(value(&r).resolution(), HtmlNoValidateResolution::FormOwner{owner,..} if owner.owner()==Some(&f.form))
    );
    let changed = context(
        &f,
        Some(HtmlFormAssociationState::EstablishedCurrent(Some(
            f.capture.subject_id(&ancestor).unwrap(),
        ))),
    );
    let other = report(&f, &changed);
    assert_eq!(value(&other).value(), Some(false));
    assert_ne!(r.qualification().context(), other.qualification().context());
    assert_eq!(r.qualification().capture(), other.qualification().capture());
}

#[test]
fn original_own_type_and_consulted_form_raw_lengths_have_exact_caps() {
    for (attrs, form, exact) in [
        (
            vec![("type", "submit"), ("formnovalidate", "formnovalidate")],
            vec![],
            14,
        ),
        (vec![("type", "submit"), ("formnovalidate", "")], vec![], 6),
        (vec![], vec![("novalidate", "novalidate")], 10),
    ] {
        let f = fixture("input", &attrs, &form, false);
        let cx = owner_context(&f);
        for cap in [exact, exact - 1] {
            let mut s = session(&f, &cx, limits(), cap, HtmlSemanticCancellation::new());
            let r = s.query(&[request(&f, HtmlSemanticQuery::SubmitterNoValidate)]);
            if cap == exact {
                assert_eq!(value(&r.unwrap()).value(), Some(true));
            } else {
                let e = r.unwrap_err();
                assert!(matches!(e.kind(), HtmlSemanticFailureKind::Attribute(_)));
                assert_eq!(s.query(&[]).unwrap_err(), e);
            }
        }
    }
}

#[test]
fn payload_records_bytes_work_and_output_caps_are_exact_and_atomic() {
    let f = fixture(
        "input",
        &[("type", "submit"), ("formnovalidate", "")],
        &[],
        false,
    );
    let cx = context(&f, None);
    let req = request(&f, HtmlSemanticQuery::SubmitterNoValidate);
    let baseline = report(&f, &cx).usage();
    assert_eq!(baseline.nested_output_records(), 1);
    assert_eq!(baseline.scratch_records(), 0);
    assert_eq!(baseline.peak_scratch_records(), 0);
    for (resource, exact) in [
        (HtmlSemanticResource::Work, baseline.work()),
        (
            HtmlSemanticResource::OutputRecords,
            baseline.output_records(),
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
                0,
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
            let mut s = session(&f, &cx, l, 100, HtmlSemanticCancellation::new());
            let r = s.query(std::slice::from_ref(&req));
            if cap == exact {
                assert_eq!(value(&r.unwrap()).value(), Some(true));
            } else {
                let e = r.unwrap_err();
                assert!(
                    matches!(e.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r==resource),
                    "{resource:?}: {e:?}"
                );
                assert_eq!(e.qualification().unwrap().limits(), l);
                assert_eq!(s.query(&[]).unwrap_err(), e);
            }
            assert_eq!(s.usage().scratch_records(), 0);
        }
    }
    let mut s = session(
        &f,
        &cx,
        HtmlSemanticLimits::new(10000000, 2, 0, 0, 1000000, 1),
        100,
        HtmlSemanticCancellation::new(),
    );
    let e = s.query(&[req.clone(), req]).unwrap_err();
    assert!(matches!(
        e.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::NestedOutputRecords,
            ..
        }
    ));
    assert_eq!(e.qualification().unwrap().context(), cx.identity());
    assert_eq!(s.query(&[]).unwrap_err(), e);
}

#[test]
fn reset_traversal_scratch_is_bounded_and_released_on_stop() {
    let f = fixture(
        "input",
        &[("form", "x")],
        &[("id", "x"), ("novalidate", "")],
        true,
    );
    let cx = context(&f, Some(HtmlFormAssociationState::ResetAgainstCapture));
    let baseline = report(&f, &cx).usage();
    assert!(baseline.peak_scratch_records() > 0);
    for cap in [
        baseline.peak_scratch_records(),
        baseline.peak_scratch_records() - 1,
    ] {
        let mut s = session(
            &f,
            &cx,
            HtmlSemanticLimits::new(10000000, 100, cap, 0, 1000000, 10000),
            100,
            HtmlSemanticCancellation::new(),
        );
        let r = s.query(&[request(&f, HtmlSemanticQuery::SubmitterNoValidate)]);
        if cap == baseline.peak_scratch_records() {
            assert_eq!(value(&r.unwrap()).value(), Some(true));
        } else {
            let e = r.unwrap_err();
            assert!(matches!(
                e.kind(),
                HtmlSemanticFailureKind::LimitExceeded {
                    resource: HtmlSemanticResource::ScratchRecords,
                    ..
                }
            ));
            assert_eq!(s.query(&[]).unwrap_err(), e);
        }
        assert_eq!(s.usage().scratch_records(), 0);
    }
}

#[test]
fn foreign_capture_and_cancellation_are_sticky_with_original_qualification() {
    let f = fixture("input", &[], &[], false);
    let cx = context(&f, None);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut s = session(&f, &cx, limits(), 100, cancel);
    let e = s
        .query(&[request(&f, HtmlSemanticQuery::SubmitterNoValidate)])
        .unwrap_err();
    assert_eq!(e.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(e.qualification().unwrap().context(), cx.identity());
    assert_eq!(e.qualification().unwrap().capture(), f.capture.identity());
    assert_eq!(s.query(&[]).unwrap_err(), e);
    let foreign = fixture("input", &[], &[], false);
    let mut s = session(&f, &cx, limits(), 100, HtmlSemanticCancellation::new());
    let e = s
        .query(&[request(&foreign, HtmlSemanticQuery::SubmitterNoValidate)])
        .unwrap_err();
    assert!(matches!(e.kind(), HtmlSemanticFailureKind::Capture(_)));
    assert_eq!(s.query(&[]).unwrap_err(), e);
}

#[test]
fn wrong_roles_and_foreign_namespaces_are_not_missing_false_control_states() {
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", &[]);
    let upper = element(&mut b, &d, "INPUT", &[("formnovalidate", "false")]);
    let text = b.add_text(&d, DocumentString::from("text")).unwrap();
    let foreign = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::svg(), None, "input"),
            attributes(&[("formnovalidate", "false")]),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let capture = b.finish().unwrap();
    let f = Fixture {
        control: capture.subject_id(&form).unwrap(),
        form: capture.subject_id(&form).unwrap(),
        capture,
    };
    let cx = context(&f, None);
    for (id, is_foreign) in [
        (f.control.clone(), false),
        (f.capture.subject_id(&upper).unwrap(), false),
        (f.capture.subject_id(&text).unwrap(), false),
        (f.capture.subject_id(&foreign).unwrap(), true),
    ] {
        let r = session(
            &f,
            &cx,
            HtmlSemanticLimits::new(10000000, 1, 0, 0, 1000000, 0),
            0,
            HtmlSemanticCancellation::new(),
        )
        .query(&[HtmlSemanticRequest::new(
            id,
            HtmlSemanticQuery::SubmitterNoValidate,
        )])
        .unwrap();
        if is_foreign {
            assert!(matches!(
                r.results()[0].fact(),
                HtmlSemanticFact::SubmitterNoValidate(HtmlSemanticValue::Unsupported(
                    HtmlSemanticUnsupported::ForeignNamespace
                ))
            ));
        } else {
            assert!(matches!(
                r.results()[0].fact(),
                HtmlSemanticFact::SubmitterNoValidate(HtmlSemanticValue::Inapplicable)
            ));
        }
        assert_eq!(r.usage().nested_output_records(), 0);
        assert_eq!(r.usage().scratch_records(), 0);
    }
}

#[test]
fn exact_attribute_names_ignore_namespaced_and_uppercase_boolean_lookalikes() {
    let (mut b, d) = builder();
    let form = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), None, "form"),
            OrderedAttributes::new([
                Attribute::new(
                    QualifiedName::new(Namespace::from_uri("urn:foreign"), None, "novalidate"),
                    "",
                ),
                Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "NOVALIDATE"),
                    "",
                ),
            ])
            .unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let control = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), None, "input"),
            OrderedAttributes::new([
                Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "type"),
                    "submit",
                ),
                Attribute::new(
                    QualifiedName::new(Namespace::from_uri("urn:foreign"), None, "formnovalidate"),
                    "",
                ),
                Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "FORMNOVALIDATE"),
                    "",
                ),
            ])
            .unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let capture = b.finish().unwrap();
    let f = Fixture {
        control: capture.subject_id(&control).unwrap(),
        form: capture.subject_id(&form).unwrap(),
        capture,
    };
    let r = report(&f, &owner_context(&f));
    let v = value(&r);
    assert_eq!(v.value(), Some(false));
    assert_eq!(v.intrinsic().intrinsic(), Some(&false));
    assert!(v.intrinsic().raw().is_none());
    assert!(v.submitter_type().is_none());
    assert_eq!(v.form_attribute().unwrap().intrinsic(), Some(&false));
    assert!(v.form_attribute().unwrap().raw().is_none());
}

#[test]
fn late_mixed_form_setting_stop_has_no_successful_prefix() {
    let f = fixture(
        "input",
        &[("type", "submit"), ("formnovalidate", "")],
        &[],
        false,
    );
    let cx = context(&f, None);
    let mut s = session(
        &f,
        &cx,
        HtmlSemanticLimits::new(10000000, 2, 0, 0, 1000000, 1),
        100,
        HtmlSemanticCancellation::new(),
    );
    let e = s
        .query(&[
            request(&f, HtmlSemanticQuery::SubmitterNoValidate),
            request(&f, HtmlSemanticQuery::SubmitterFormMethod),
        ])
        .unwrap_err();
    assert!(matches!(
        e.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::NestedOutputRecords,
            ..
        }
    ));
    assert_eq!(e.qualification().unwrap().context(), cx.identity());
    assert_eq!(s.usage().scratch_records(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), e);
}
