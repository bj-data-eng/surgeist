use crate::semantics::support::{
    moderate_semantic_limits as limits, small_builder_without_registries as builder,
};
use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlControlFacts, HtmlControlFactsLimits,
    HtmlFormAssociation, HtmlFormAssociationState, HtmlFormOwnerOrigin, HtmlSemanticCancellation,
    HtmlSemanticContext, HtmlSemanticContextInput, HtmlSemanticFact, HtmlSemanticFailureKind,
    HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport,
    HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticSession, HtmlSemanticUnsupported,
    HtmlSemanticValue, HtmlSubmitterTypeObservation,
};
use surgeist_html_semantics::{
    HtmlFormEnctypeSelection, HtmlFormMethodSelection, HtmlSubmitterFormEnctype,
    HtmlSubmitterFormMethod,
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
fn both(f: &Fixture, cx: &HtmlSemanticContext) -> HtmlSemanticReport {
    session(f, cx, limits(), 100000, HtmlSemanticCancellation::new())
        .query(&[
            request(f, HtmlSemanticQuery::SubmitterFormMethod),
            request(f, HtmlSemanticQuery::SubmitterFormEnctype),
        ])
        .unwrap()
}
fn method(r: &HtmlSemanticReport) -> &HtmlSubmitterFormMethod {
    match r.results()[0].fact() {
        HtmlSemanticFact::SubmitterFormMethod(HtmlSemanticValue::Known(v)) => v,
        other => panic!("expected method observation: {other:?}"),
    }
}
fn enctype(r: &HtmlSemanticReport) -> &HtmlSubmitterFormEnctype {
    match r.results()[1].fact() {
        HtmlSemanticFact::SubmitterFormEnctype(HtmlSemanticValue::Known(v)) => v,
        other => panic!("expected enctype observation: {other:?}"),
    }
}
fn assert_owner(
    r: &HtmlSemanticReport,
    f: &Fixture,
    m: HtmlFormMethod,
    e: HtmlFormEnctype,
    origin: HtmlFormOwnerOrigin,
) {
    assert!(
        matches!(method(r).selection(), HtmlFormMethodSelection::Owner{value,owner}
        if *value==m && owner.owner()==Some(&f.form) && owner.origin()==origin)
    );
    assert!(
        matches!(enctype(r).selection(), HtmlFormEnctypeSelection::Owner{value,owner}
        if *value==e && owner.owner()==Some(&f.form) && owner.origin()==origin)
    );
}
fn assert_override(r: &HtmlSemanticReport, m: HtmlFormMethod, e: HtmlFormEnctype) {
    assert_eq!(method(r).selection(), &HtmlFormMethodSelection::Override(m));
    assert_eq!(
        enctype(r).selection(),
        &HtmlFormEnctypeSelection::Override(e)
    );
    assert!(method(r).form_attribute().is_none());
    assert!(enctype(r).form_attribute().is_none());
}

#[test]
fn submit_and_image_overrides_win_without_association_or_ancestry() {
    for state in ["submit", "image"] {
        let f = fixture(
            "input",
            &[
                ("type", state),
                ("formmethod", "POST"),
                ("formenctype", "MULTIPART/FORM-DATA"),
            ],
            &[("method", "dialog")],
            false,
        );
        let cx = context(&f, None);
        let r = both(&f, &cx);
        assert_override(&r, HtmlFormMethod::Post, HtmlFormEnctype::MultipartFormData);
        assert_eq!(
            method(&r).intrinsic().raw(),
            Some(&DocumentString::from("POST"))
        );
        assert_eq!(
            method(&r).intrinsic().context_author(),
            HtmlAttributeContextAuthorState::Conforming
        );
        assert!(matches!(
            method(&r).submitter_type(),
            Some(HtmlSubmitterTypeObservation::Input(_))
        ));
    }
}

#[test]
fn present_empty_or_malformed_override_defaults_locally_instead_of_owner() {
    for raw in ["", " post ", "unknown", "\u{a0}post"] {
        let f = fixture(
            "input",
            &[
                ("type", "submit"),
                ("formmethod", raw),
                ("formenctype", raw),
            ],
            &[("method", "post"), ("enctype", "text/plain")],
            true,
        );
        let r = both(&f, &owner_context(&f));
        assert_override(&r, HtmlFormMethod::Get, HtmlFormEnctype::UrlEncoded);
        assert_eq!(
            method(&r).intrinsic().raw(),
            Some(&DocumentString::from(raw))
        );
        assert_eq!(
            enctype(&r).intrinsic().raw(),
            Some(&DocumentString::from(raw))
        );
        assert_eq!(
            method(&r).intrinsic().recognition(),
            if raw.is_empty() {
                HtmlAttributeRecognition::Empty
            } else {
                HtmlAttributeRecognition::Unrecognized
            }
        );
        assert!(matches!(
            method(&r).intrinsic().author(),
            HtmlAttributeAuthorState::Nonconforming(_)
        ));
    }
}

#[test]
fn absent_overrides_consult_owner_and_preserve_absence() {
    let f = fixture(
        "button",
        &[],
        &[("method", "DIALOG"), ("enctype", "TEXT/PLAIN")],
        false,
    );
    let r = both(&f, &owner_context(&f));
    assert_owner(
        &r,
        &f,
        HtmlFormMethod::Dialog,
        HtmlFormEnctype::PlainText,
        HtmlFormOwnerOrigin::EstablishedCurrent,
    );
    assert!(method(&r).intrinsic().raw().is_none());
    assert!(method(&r).intrinsic().intrinsic().is_none());
    assert!(enctype(&r).intrinsic().intrinsic().is_none());
    assert!(method(&r).submitter_type().is_none());
    assert!(enctype(&r).submitter_type().is_none());
    assert_eq!(
        method(&r).form_attribute().unwrap().raw(),
        Some(&DocumentString::from("DIALOG"))
    );
}

#[test]
fn absent_empty_invalid_owner_attributes_have_their_own_defaults() {
    for raw in [None, Some(""), Some("unknown"), Some(" post ")] {
        let attrs = raw
            .map(|v| vec![("method", v), ("enctype", v)])
            .unwrap_or_default();
        let f = fixture("input", &[], &attrs, false);
        let r = both(&f, &owner_context(&f));
        assert_owner(
            &r,
            &f,
            HtmlFormMethod::Get,
            HtmlFormEnctype::UrlEncoded,
            HtmlFormOwnerOrigin::EstablishedCurrent,
        );
        assert_eq!(
            method(&r).form_attribute().unwrap().raw().cloned(),
            raw.map(DocumentString::from)
        );
    }
}

#[test]
fn non_submit_inputs_use_owner_and_retain_forbidden_override_authorship() {
    for (state, expected) in [
        (None, HtmlInputType::Text),
        (Some("unknown"), HtmlInputType::Text),
        (Some("text"), HtmlInputType::Text),
        (Some("reset"), HtmlInputType::Reset),
        (Some("button"), HtmlInputType::Button),
        (Some("file"), HtmlInputType::File),
    ] {
        let mut attrs = vec![("formmethod", "get"), ("formenctype", "text/plain")];
        if let Some(state) = state {
            attrs.push(("type", state));
        }
        let f = fixture(
            "input",
            &attrs,
            &[("method", "post"), ("enctype", "multipart/form-data")],
            false,
        );
        let r = both(&f, &owner_context(&f));
        assert_owner(
            &r,
            &f,
            HtmlFormMethod::Post,
            HtmlFormEnctype::MultipartFormData,
            HtmlFormOwnerOrigin::EstablishedCurrent,
        );
        assert_eq!(
            method(&r).intrinsic().intrinsic(),
            Some(&HtmlFormMethod::Get)
        );
        assert_eq!(
            method(&r).intrinsic().context_author(),
            HtmlAttributeContextAuthorState::Nonconforming(
                HtmlAttributeContextConformanceError::InputStateForbidsAttribute
            )
        );
        assert!(
            matches!(method(&r).submitter_type(), Some(HtmlSubmitterTypeObservation::Input(v))
            if v.intrinsic()==Some(&expected))
        );
    }
}

#[test]
fn explicit_button_submit_ignores_command_and_select_parent() {
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", &[]);
    let select = element(&mut b, &d, "select", &[]);
    let control = element(
        &mut b,
        &d,
        "button",
        &[
            ("type", "submit"),
            ("command", ""),
            ("commandfor", "missing"),
            ("formmethod", "dialog"),
            ("formenctype", "text/plain"),
        ],
    );
    b.establish_children(&select, std::slice::from_ref(&control))
        .unwrap();
    let capture = b.finish().unwrap();
    let f = Fixture {
        control: capture.subject_id(&control).unwrap(),
        form: capture.subject_id(&form).unwrap(),
        capture,
    };
    let r = both(&f, &context(&f, None));
    assert_override(&r, HtmlFormMethod::Dialog, HtmlFormEnctype::PlainText);
    assert_eq!(
        method(&r).intrinsic().button_submitter_context(),
        Some(HtmlButtonSubmitterContext::SubmitButton)
    );
}

#[test]
fn button_and_reset_types_ignore_overrides_without_parent_evidence() {
    for state in ["button", "reset"] {
        let f = fixture(
            "button",
            &[
                ("type", state),
                ("formmethod", "get"),
                ("formenctype", "text/plain"),
            ],
            &[("method", "post"), ("enctype", "multipart/form-data")],
            false,
        );
        let r = both(&f, &owner_context(&f));
        assert_owner(
            &r,
            &f,
            HtmlFormMethod::Post,
            HtmlFormEnctype::MultipartFormData,
            HtmlFormOwnerOrigin::EstablishedCurrent,
        );
        assert_eq!(
            method(&r).intrinsic().button_submitter_context(),
            Some(HtmlButtonSubmitterContext::NotSubmitButton)
        );
        assert_eq!(
            method(&r).intrinsic().context_author(),
            HtmlAttributeContextAuthorState::Nonconforming(
                HtmlAttributeContextConformanceError::FormOverrideRequiresSubmitButton
            )
        );
    }
}

#[test]
fn auto_buttons_under_select_are_non_submit_regardless_child_position() {
    for first in [true, false] {
        let (mut b, d) = builder();
        let form = element(
            &mut b,
            &d,
            "form",
            &[("method", "post"), ("enctype", "text/plain")],
        );
        let select = element(&mut b, &d, "select", &[]);
        let other = element(&mut b, &d, "span", &[]);
        let control = element(
            &mut b,
            &d,
            "button",
            &[
                ("type", "invalid"),
                ("formmethod", "get"),
                ("formenctype", "multipart/form-data"),
            ],
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
        let r = both(&f, &owner_context(&f));
        assert_owner(
            &r,
            &f,
            HtmlFormMethod::Post,
            HtmlFormEnctype::PlainText,
            HtmlFormOwnerOrigin::EstablishedCurrent,
        );
        assert_eq!(
            method(&r).intrinsic().button_submitter_context(),
            Some(HtmlButtonSubmitterContext::NotSubmitButton)
        );
    }
}

#[test]
fn auto_command_presence_proves_non_submit_without_value_or_parent_reads() {
    let huge = "x".repeat(1000);
    for name in ["command", "commandfor"] {
        let f = fixture(
            "button",
            &[(name, &huge), ("formmethod", "get"), ("formenctype", "bad")],
            &[("method", "post"), ("enctype", "text/plain")],
            false,
        );
        let cx = owner_context(&f);
        let r = session(&f, &cx, limits(), 10, HtmlSemanticCancellation::new())
            .query(&[
                request(&f, HtmlSemanticQuery::SubmitterFormMethod),
                request(&f, HtmlSemanticQuery::SubmitterFormEnctype),
            ])
            .unwrap();
        assert_owner(
            &r,
            &f,
            HtmlFormMethod::Post,
            HtmlFormEnctype::PlainText,
            HtmlFormOwnerOrigin::EstablishedCurrent,
        );
    }
}

#[test]
fn auto_root_or_non_select_parent_proves_submit_with_no_association() {
    for root in [false, true] {
        let (mut b, d) = builder();
        let form = element(&mut b, &d, "form", &[]);
        let parent = element(&mut b, &d, "div", &[]);
        let control = element(
            &mut b,
            &d,
            "button",
            &[("formmethod", "post"), ("formenctype", "text/plain")],
        );
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
        assert_override(
            &both(&f, &context(&f, None)),
            HtmlFormMethod::Post,
            HtmlFormEnctype::PlainText,
        );
    }
}

#[test]
fn unknown_auto_parent_keeps_unchecked_raw_and_missing_selection() {
    let f = fixture(
        "button",
        &[("formmethod", "post"), ("formenctype", "text/plain")],
        &[("method", "get")],
        false,
    );
    let r = both(&f, &owner_context(&f));
    let missing = HtmlSemanticMissing::Parent(f.control.clone());
    assert_eq!(
        method(&r).selection(),
        &HtmlFormMethodSelection::Missing(missing.clone())
    );
    assert_eq!(
        enctype(&r).selection(),
        &HtmlFormEnctypeSelection::Missing(missing)
    );
    assert_eq!(
        method(&r).intrinsic().context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
    assert_eq!(
        method(&r).intrinsic().raw(),
        Some(&DocumentString::from("post"))
    );
    assert!(method(&r).form_attribute().is_none());
}

#[test]
fn null_owner_is_not_an_invented_default_or_missing_association() {
    let f = fixture("input", &[], &[], false);
    let cx = context(&f, Some(HtmlFormAssociationState::EstablishedCurrent(None)));
    let r = both(&f, &cx);
    assert!(
        matches!(method(&r).selection(), HtmlFormMethodSelection::NoFormOwner(v)
        if v.owner().is_none() && v.origin()==HtmlFormOwnerOrigin::EstablishedCurrent)
    );
    assert!(
        matches!(enctype(&r).selection(), HtmlFormEnctypeSelection::NoFormOwner(v)
        if v.owner().is_none() && v.origin()==HtmlFormOwnerOrigin::EstablishedCurrent)
    );
    assert!(method(&r).form_attribute().is_none());
    for state in [None, Some(HtmlFormAssociationState::Unavailable)] {
        let cx = context(&f, state);
        let r = both(&f, &cx);
        assert_eq!(
            method(&r).selection(),
            &HtmlFormMethodSelection::Missing(HtmlSemanticMissing::FormAssociation(
                f.control.clone()
            ))
        );
        assert_eq!(
            enctype(&r).selection(),
            &HtmlFormEnctypeSelection::Missing(HtmlSemanticMissing::FormAssociation(
                f.control.clone()
            ))
        );
    }
}

#[test]
fn reset_owner_reuses_source_algorithm_and_preserves_origin_or_missing() {
    let f = fixture(
        "input",
        &[],
        &[("method", "post"), ("enctype", "text/plain")],
        true,
    );
    let cx = context(&f, Some(HtmlFormAssociationState::ResetAgainstCapture));
    assert_owner(
        &both(&f, &cx),
        &f,
        HtmlFormMethod::Post,
        HtmlFormEnctype::PlainText,
        HtmlFormOwnerOrigin::ResetAgainstCapture,
    );
    let f = fixture("input", &[], &[], false);
    let cx = context(&f, Some(HtmlFormAssociationState::ResetAgainstCapture));
    let r = both(&f, &cx);
    assert_eq!(
        method(&r).selection(),
        &HtmlFormMethodSelection::Missing(HtmlSemanticMissing::Parent(f.control.clone()))
    );
}

#[test]
fn absent_override_skips_oversized_type_and_present_override_skips_form_raw() {
    let huge = "x".repeat(1000);
    let f = fixture(
        "button",
        &[("type", &huge)],
        &[("method", "post"), ("enctype", "text/plain")],
        false,
    );
    let cx = owner_context(&f);
    let r = session(&f, &cx, limits(), 10, HtmlSemanticCancellation::new())
        .query(&[
            request(&f, HtmlSemanticQuery::SubmitterFormMethod),
            request(&f, HtmlSemanticQuery::SubmitterFormEnctype),
        ])
        .unwrap();
    assert_owner(
        &r,
        &f,
        HtmlFormMethod::Post,
        HtmlFormEnctype::PlainText,
        HtmlFormOwnerOrigin::EstablishedCurrent,
    );
    assert!(method(&r).submitter_type().is_none());
    let f = fixture(
        "input",
        &[
            ("type", "submit"),
            ("formmethod", "post"),
            ("formenctype", "bad"),
        ],
        &[("method", &huge), ("enctype", &huge)],
        false,
    );
    let cx = owner_context(&f);
    let r = session(&f, &cx, limits(), 6, HtmlSemanticCancellation::new())
        .query(&[
            request(&f, HtmlSemanticQuery::SubmitterFormMethod),
            request(&f, HtmlSemanticQuery::SubmitterFormEnctype),
        ])
        .unwrap();
    assert_override(&r, HtmlFormMethod::Post, HtmlFormEnctype::UrlEncoded);
}

#[test]
fn exact_utf16_surrogate_overrides_preserve_raw_and_default_locally() {
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", &[]);
    let raw = DocumentString::from_utf16(vec![0x70, 0x6f, 0x73, 0x74, 0xd800]);
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
                    QualifiedName::new(Namespace::none(), Some("kept".into()), "formmethod"),
                    raw.clone(),
                ),
                Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "formenctype"),
                    raw.clone(),
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
    let r = both(&f, &context(&f, None));
    assert_override(&r, HtmlFormMethod::Get, HtmlFormEnctype::UrlEncoded);
    assert_eq!(method(&r).intrinsic().raw(), Some(&raw));
    assert_eq!(enctype(&r).intrinsic().raw(), Some(&raw));
    assert_eq!(
        method(&r)
            .intrinsic()
            .raw_attribute()
            .unwrap()
            .name()
            .prefix(),
        Some(&DocumentString::from("kept"))
    );
}

#[test]
fn actual_nonancestor_owner_and_context_change_select_fresh_qualified_values() {
    let (mut b, d) = builder();
    let ancestor = element(
        &mut b,
        &d,
        "form",
        &[("method", "get"), ("enctype", "text/plain")],
    );
    let actual = element(
        &mut b,
        &d,
        "form",
        &[("method", "post"), ("enctype", "multipart/form-data")],
    );
    let control = element(&mut b, &d, "input", &[]);
    b.establish_children(&ancestor, std::slice::from_ref(&control))
        .unwrap();
    b.establish_children(&actual, &[]).unwrap();
    b.establish_children(&d, &[ancestor, actual.clone()])
        .unwrap();
    let capture = b.finish().unwrap();
    let f = Fixture {
        control: capture.subject_id(&control).unwrap(),
        form: capture.subject_id(&actual).unwrap(),
        capture,
    };
    let current = owner_context(&f);
    let r = both(&f, &current);
    assert_owner(
        &r,
        &f,
        HtmlFormMethod::Post,
        HtmlFormEnctype::MultipartFormData,
        HtmlFormOwnerOrigin::EstablishedCurrent,
    );
    let null = context(&f, Some(HtmlFormAssociationState::EstablishedCurrent(None)));
    let other = both(&f, &null);
    assert!(matches!(
        method(&other).selection(),
        HtmlFormMethodSelection::NoFormOwner(_)
    ));
    assert_ne!(r.qualification().context(), other.qualification().context());
    assert_eq!(r.qualification().capture(), other.qualification().capture());
}

#[test]
fn original_type_override_and_consulted_form_lengths_are_admitted() {
    for (attrs, form, raw_cap) in [
        (vec![("type", "submit"), ("formmethod", "post")], vec![], 5),
        (
            vec![("type", "submit"), ("formmethod", " post ")],
            vec![],
            5,
        ),
        (vec![], vec![("method", "dialog")], 5),
    ] {
        let f = fixture("input", &attrs, &form, false);
        let cx = owner_context(&f);
        let mut s = session(&f, &cx, limits(), raw_cap, HtmlSemanticCancellation::new());
        let e = s
            .query(&[request(&f, HtmlSemanticQuery::SubmitterFormMethod)])
            .unwrap_err();
        assert!(matches!(e.kind(), HtmlSemanticFailureKind::Attribute(_)));
        assert_eq!(e.qualification().unwrap().capture(), f.capture.identity());
        assert_eq!(s.query(&[]).unwrap_err(), e);
    }
    let f = fixture(
        "input",
        &[("type", "submit"), ("formmethod", "post")],
        &[],
        false,
    );
    let cx = context(&f, None);
    let r = session(&f, &cx, limits(), 6, HtmlSemanticCancellation::new())
        .query(&[request(&f, HtmlSemanticQuery::SubmitterFormMethod)])
        .unwrap();
    assert_eq!(
        method(&r).selection(),
        &HtmlFormMethodSelection::Override(HtmlFormMethod::Post)
    );
}

#[test]
fn both_setting_payloads_use_exact_finite_record_byte_work_and_output_budgets() {
    for query in [
        HtmlSemanticQuery::SubmitterFormMethod,
        HtmlSemanticQuery::SubmitterFormEnctype,
    ] {
        let f = fixture(
            "input",
            &[
                ("type", "submit"),
                ("formmethod", "post"),
                ("formenctype", "text/plain"),
            ],
            &[],
            false,
        );
        let cx = context(&f, None);
        let req = request(&f, query);
        let baseline = session(&f, &cx, limits(), 100, HtmlSemanticCancellation::new())
            .query(std::slice::from_ref(&req))
            .unwrap()
            .usage();
        assert_eq!(baseline.nested_output_records(), 1);
        assert_eq!(baseline.scratch_records(), 0);
        assert_eq!(baseline.peak_scratch_records(), 0);
        assert!(baseline.allocated_bytes() > 0);
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
                    assert!(r.is_ok(), "{query:?} {resource:?}: {r:?}");
                } else {
                    let e = r.unwrap_err();
                    assert!(
                        matches!(e.kind(), HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r==resource),
                        "{query:?} {resource:?}: {e:?}"
                    );
                    assert_eq!(e.qualification().unwrap().limits(), l);
                    assert_eq!(s.query(&[]).unwrap_err(), e);
                }
                assert_eq!(s.usage().scratch_records(), 0);
            }
        }
    }
}

#[test]
fn late_setting_batch_failure_discards_results_and_retains_original_identity() {
    let f = fixture(
        "input",
        &[
            ("type", "submit"),
            ("formmethod", "post"),
            ("formenctype", "text/plain"),
        ],
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
            request(&f, HtmlSemanticQuery::SubmitterFormMethod),
            request(&f, HtmlSemanticQuery::SubmitterFormEnctype),
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
    assert_eq!(e.qualification().unwrap().capture(), f.capture.identity());
    assert_eq!(s.query(&[]).unwrap_err(), e);
}

#[test]
fn cancellation_and_foreign_subject_are_sticky_qualified_failures() {
    let f = fixture("input", &[], &[], false);
    let cx = context(&f, None);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut s = session(&f, &cx, limits(), 100, cancel);
    let e = s
        .query(&[request(&f, HtmlSemanticQuery::SubmitterFormMethod)])
        .unwrap_err();
    assert_eq!(e.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(e.qualification().unwrap().context(), cx.identity());
    assert_eq!(s.query(&[]).unwrap_err(), e);
    let foreign = fixture("input", &[], &[], false);
    let mut s = session(&f, &cx, limits(), 100, HtmlSemanticCancellation::new());
    let e = s
        .query(&[request(&foreign, HtmlSemanticQuery::SubmitterFormEnctype)])
        .unwrap_err();
    assert!(matches!(e.kind(), HtmlSemanticFailureKind::Capture(_)));
    assert_eq!(s.query(&[]).unwrap_err(), e);
}

#[test]
fn wrong_roles_foreign_namespaces_and_non_elements_need_no_setting_storage() {
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", &[]);
    let text = b.add_text(&d, DocumentString::from("text")).unwrap();
    let foreign = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::svg(), None, "input"),
            attributes(&[("formmethod", "post")]),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let upper = element(&mut b, &d, "INPUT", &[("formmethod", "post")]);
    let capture = b.finish().unwrap();
    let f = Fixture {
        control: capture.subject_id(&form).unwrap(),
        form: capture.subject_id(&form).unwrap(),
        capture,
    };
    let cx = context(&f, None);
    for (id, is_foreign) in [
        (f.control.clone(), false),
        (f.capture.subject_id(&text).unwrap(), false),
        (f.capture.subject_id(&upper).unwrap(), false),
        (f.capture.subject_id(&foreign).unwrap(), true),
    ] {
        let r = session(
            &f,
            &cx,
            HtmlSemanticLimits::new(10000000, 2, 0, 0, 1000000, 0),
            0,
            HtmlSemanticCancellation::new(),
        )
        .query(&[
            HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::SubmitterFormMethod),
            HtmlSemanticRequest::new(id, HtmlSemanticQuery::SubmitterFormEnctype),
        ])
        .unwrap();
        if is_foreign {
            assert!(matches!(
                r.results()[0].fact(),
                HtmlSemanticFact::SubmitterFormMethod(HtmlSemanticValue::Unsupported(
                    HtmlSemanticUnsupported::ForeignNamespace
                ))
            ));
            assert!(matches!(
                r.results()[1].fact(),
                HtmlSemanticFact::SubmitterFormEnctype(HtmlSemanticValue::Unsupported(
                    HtmlSemanticUnsupported::ForeignNamespace
                ))
            ));
        } else {
            assert!(matches!(
                r.results()[0].fact(),
                HtmlSemanticFact::SubmitterFormMethod(HtmlSemanticValue::Inapplicable)
            ));
            assert!(matches!(
                r.results()[1].fact(),
                HtmlSemanticFact::SubmitterFormEnctype(HtmlSemanticValue::Inapplicable)
            ));
        }
        assert_eq!(r.usage().nested_output_records(), 0);
        assert_eq!(r.usage().scratch_records(), 0);
    }
}

#[test]
fn present_submit_overrides_bypass_known_null_and_reset_root_preserves_null_origin() {
    let f = fixture(
        "input",
        &[
            ("type", "submit"),
            ("formmethod", "post"),
            ("formenctype", "text/plain"),
        ],
        &[],
        false,
    );
    let cx = context(&f, Some(HtmlFormAssociationState::EstablishedCurrent(None)));
    assert_override(
        &both(&f, &cx),
        HtmlFormMethod::Post,
        HtmlFormEnctype::PlainText,
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
    let r = both(&f, &cx);
    assert!(
        matches!(method(&r).selection(), HtmlFormMethodSelection::NoFormOwner(owner)
        if owner.owner().is_none() && owner.origin()==HtmlFormOwnerOrigin::ResetAgainstCapture)
    );
    assert!(
        matches!(enctype(&r).selection(), HtmlFormEnctypeSelection::NoFormOwner(owner)
        if owner.owner().is_none() && owner.origin()==HtmlFormOwnerOrigin::ResetAgainstCapture)
    );
}

#[test]
fn exact_expanded_override_names_ignore_namespaced_and_uppercase_lookalikes() {
    let (mut b, d) = builder();
    let form = element(
        &mut b,
        &d,
        "form",
        &[("method", "dialog"), ("enctype", "multipart/form-data")],
    );
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
                    QualifiedName::new(
                        Namespace::from_uri("http://www.w3.org/XML/1998/namespace"),
                        None,
                        "formmethod",
                    ),
                    "post",
                ),
                Attribute::new(
                    QualifiedName::new(
                        Namespace::from_uri("http://www.w3.org/XML/1998/namespace"),
                        None,
                        "formenctype",
                    ),
                    "text/plain",
                ),
                Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "FORMMETHOD"),
                    "get",
                ),
                Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "FORMENCTYPE"),
                    "text/plain",
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
    let r = both(&f, &owner_context(&f));
    assert_owner(
        &r,
        &f,
        HtmlFormMethod::Dialog,
        HtmlFormEnctype::MultipartFormData,
        HtmlFormOwnerOrigin::EstablishedCurrent,
    );
    assert!(method(&r).intrinsic().raw().is_none());
    assert!(enctype(&r).intrinsic().raw().is_none());
    assert!(method(&r).submitter_type().is_none());
}
