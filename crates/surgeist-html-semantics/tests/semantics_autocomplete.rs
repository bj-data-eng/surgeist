use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::{
    HtmlAutocompleteDefaultResolution, HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject,
    HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlCaptureSubjectId, HtmlCapturedIsValue,
    HtmlControlAutocomplete, HtmlControlFacts, HtmlControlFactsLimits, HtmlFormAssociation,
    HtmlFormAssociationState, HtmlFormOwnerOrigin, HtmlScriptingStatus, HtmlSemanticCancellation,
    HtmlSemanticContext, HtmlSemanticContextInput, HtmlSemanticFact, HtmlSemanticFailureKind,
    HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport,
    HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticUnsupported, HtmlSemanticValue,
};
#[path = "common/session.rs"]
mod shared_session;
use shared_session::session;
use surgeist_html::*;

fn attributes(values: &[(&str, &str)]) -> OrderedAttributes {
    OrderedAttributes::new(values.iter().map(|(name, value)| {
        Attribute::new(QualifiedName::new(Namespace::none(), None, *name), *value)
    }))
    .unwrap()
}
fn builder() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 1000, 100, 100000, 100, 0, 0, 16_777_216),
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
    control: HtmlCaptureSubjectId,
    form: HtmlCaptureSubjectId,
}
fn fixture(local: &str, attrs: &[(&str, &str)], form_attrs: &[(&str, &str)]) -> Fixture {
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", form_attrs);
    let control = element(&mut b, &d, local, attrs);
    b.establish_children(&form, std::slice::from_ref(&control))
        .unwrap();
    b.establish_children(&d, std::slice::from_ref(&form))
        .unwrap();
    let c = b.finish().unwrap();
    Fixture {
        control: c.subject_id(&control).unwrap(),
        form: c.subject_id(&form).unwrap(),
        capture: c,
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
fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(10000000, 100, 1000, 0, 1000000, 10000)
}

fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::ControlAutocomplete)
}
fn report(f: &Fixture, cx: &HtmlSemanticContext) -> HtmlSemanticReport {
    session(
        &f.capture,
        cx,
        limits(),
        100000,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(&f.control)])
    .unwrap()
}
fn value(r: &HtmlSemanticReport) -> &HtmlControlAutocomplete {
    match r.results()[0].fact() {
        HtmlSemanticFact::ControlAutocomplete(HtmlSemanticValue::Known(v)) => v,
        other => panic!("expected autocomplete observation, got {other:?}"),
    }
}
fn setting(v: &HtmlControlAutocomplete) -> HtmlFormAutocomplete {
    match v.default_resolution() {
        HtmlAutocompleteDefaultResolution::Resolved { setting, .. } => *setting,
        other => panic!("expected default resolution, got {other:?}"),
    }
}
fn default(v: &HtmlControlAutocomplete) -> &HtmlAutocompleteDefault {
    match v.intrinsic().processing() {
        HtmlAutocompleteProcessingState::Defaulted(v) => v,
        other => panic!("expected default, got {other:?}"),
    }
}

#[test]
fn form_default_uses_on_off_and_missing_invalid_states_without_changing_empty_idl() {
    for (form_attrs, expected) in [
        (&[("autocomplete", "OFF")][..], HtmlFormAutocomplete::Off),
        (&[("autocomplete", "on")][..], HtmlFormAutocomplete::On),
        (&[][..], HtmlFormAutocomplete::On),
        (&[("autocomplete", "")][..], HtmlFormAutocomplete::On),
        (&[("autocomplete", " off ")][..], HtmlFormAutocomplete::On),
        (&[("autocomplete", "bogus")][..], HtmlFormAutocomplete::On),
    ] {
        for attrs in [
            &[][..],
            &[("autocomplete", "")][..],
            &[("autocomplete", "nonsense")][..],
        ] {
            let f = fixture("input", attrs, form_attrs);
            let cx = context(
                &f,
                Some(HtmlFormAssociationState::EstablishedCurrent(Some(
                    f.form.clone(),
                ))),
            );
            let r = report(&f, &cx);
            let v = value(&r);
            assert_eq!(setting(v), expected);
            assert!(default(v).idl_value().is_empty());
            assert!(default(v).scope().tokens().is_empty());
            assert_eq!(
                v.credential_type(),
                Some(HtmlAutocompleteCredentialOutcome::NotAssigned)
            );
            assert_eq!(
                v.intrinsic().raw().map(DocumentString::code_units),
                attrs
                    .first()
                    .map(|(_, v)| DocumentString::from(*v))
                    .as_ref()
                    .map(DocumentString::code_units)
            );
            let owner = v.form_autocomplete().unwrap();
            assert_eq!(owner.intrinsic(), Some(&expected));
            let author = match form_attrs.first().map(|(_, value)| *value) {
                None => HtmlAttributeAuthorState::Missing,
                Some("OFF" | "on") => HtmlAttributeAuthorState::Conforming,
                _ => HtmlAttributeAuthorState::Nonconforming(
                    HtmlAttributeConformanceError::UnrecognizedKeyword,
                ),
            };
            assert_eq!(owner.author(), author);
            assert_eq!(owner.inventory_rows(), [13]);
            assert_eq!(v.intrinsic().inventory_rows(), [14]);
            assert_eq!(
                owner.raw().map(DocumentString::code_units),
                form_attrs
                    .first()
                    .map(|(_, v)| DocumentString::from(*v))
                    .as_ref()
                    .map(DocumentString::code_units)
            );
        }
    }
}
#[test]
fn actual_null_owner_is_on_but_unavailable_association_preserves_missing_and_raw() {
    let f = fixture(
        "textarea",
        &[("autocomplete", "invalid")],
        &[("autocomplete", "off")],
    );
    let null = context(&f, Some(HtmlFormAssociationState::EstablishedCurrent(None)));
    let r = report(&f, &null);
    let v = value(&r);
    assert_eq!(setting(v), HtmlFormAutocomplete::On);
    assert!(v.form_autocomplete().is_none());
    assert!(
        matches!(v.default_resolution(),HtmlAutocompleteDefaultResolution::Resolved{owner,..} if owner.owner().is_none() && owner.origin()==HtmlFormOwnerOrigin::EstablishedCurrent)
    );
    for state in [None, Some(HtmlFormAssociationState::Unavailable)] {
        let cx = context(&f, state);
        let r = report(&f, &cx);
        let v = value(&r);
        assert_eq!(
            v.default_resolution(),
            &HtmlAutocompleteDefaultResolution::Missing(HtmlSemanticMissing::FormAssociation(
                f.control.clone()
            ))
        );
        assert_eq!(
            v.intrinsic().raw().unwrap(),
            &DocumentString::from("invalid")
        );
        assert!(v.form_autocomplete().is_none());
        assert!(default(v).idl_value().is_empty());
    }
}
#[test]
fn successful_token_branches_need_neither_owner_nor_form_raw_admission() {
    for (raw, credential) in [
        ("on", HtmlAutocompleteCredentialOutcome::NotAssigned),
        ("OFF", HtmlAutocompleteCredentialOutcome::NotAssigned),
        ("NAME", HtmlAutocompleteCredentialOutcome::Null),
        (
            "username webauthn",
            HtmlAutocompleteCredentialOutcome::Credential(HtmlAutocompleteCredential::Webauthn),
        ),
    ] {
        let f = fixture(
            "input",
            &[("autocomplete", raw)],
            &[("autocomplete", "this form value exceeds the control cap")],
        );
        let cx = context(&f, None);
        let r = session(
            &f.capture,
            &cx,
            limits(),
            raw.len(),
            HtmlSemanticCancellation::new(),
        )
        .query(&[request(&f.control)])
        .unwrap();
        let v = value(&r);
        assert_eq!(
            v.default_resolution(),
            &HtmlAutocompleteDefaultResolution::NotRequired
        );
        assert_eq!(v.credential_type(), Some(credential));
        assert!(v.form_autocomplete().is_none());
        assert!(matches!(
            v.intrinsic().processing(),
            HtmlAutocompleteProcessingState::Processed(_)
        ));
    }
}
#[test]
fn hidden_anchor_default_does_not_consult_association_and_keeps_field_empty() {
    let f = fixture(
        "input",
        &[("type", "hidden"), ("autocomplete", "off")],
        &[("autocomplete", "this form value is not admitted")],
    );
    let cx = context(&f, None);
    let r = session(
        &f.capture,
        &cx,
        limits(),
        6,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(&f.control)])
    .unwrap();
    let v = value(&r);
    assert_eq!(
        v.default_resolution(),
        &HtmlAutocompleteDefaultResolution::AnchorEmpty
    );
    assert_eq!(default(v).field(), HtmlAutocompleteDefaultField::Empty);
    assert!(default(v).idl_value().is_empty());
    assert!(v.form_autocomplete().is_none());
    assert_eq!(
        v.credential_type(),
        Some(HtmlAutocompleteCredentialOutcome::NotAssigned)
    );
}
#[test]
fn captured_type_default_and_actual_prohibited_types_are_not_guessed() {
    for attrs in [&[][..], &[("type", "invalid")][..], &[("type", "")][..]] {
        let f = fixture("input", attrs, &[]);
        let cx = context(&f, Some(HtmlFormAssociationState::EstablishedCurrent(None)));
        let r = report(&f, &cx);
        let v = value(&r);
        assert_eq!(
            v.input_type().unwrap().intrinsic(),
            Some(&HtmlInputType::Text)
        );
        assert_eq!(setting(v), HtmlFormAutocomplete::On);
    }
    for (raw, expected) in [
        ("file", HtmlInputType::File),
        ("checkbox", HtmlInputType::Checkbox),
        ("button", HtmlInputType::Button),
    ] {
        let f = fixture("input", &[("type", raw), ("autocomplete", "name")], &[]);
        let cx = context(&f, None);
        let r = report(&f, &cx);
        let v = value(&r);
        assert_eq!(
            v.intrinsic().processing(),
            &HtmlAutocompleteProcessingState::InapplicableInputState(expected)
        );
        assert_eq!(v.input_type().unwrap().intrinsic(), Some(&expected));
        assert_eq!(
            v.default_resolution(),
            &HtmlAutocompleteDefaultResolution::NotRequired
        );
        assert_eq!(v.credential_type(), None);
    }
}
#[test]
fn select_and_textarea_use_the_shared_intrinsic_mantle_and_report_without_type_fact() {
    for local in ["select", "textarea"] {
        let f = fixture(local, &[("autocomplete", "name")], &[]);
        let cx = context(&f, None);
        let r = report(&f, &cx);
        let v = value(&r);
        assert!(v.input_type().is_none());
        assert_eq!(
            v.intrinsic().mantle(),
            Some(HtmlAutocompleteMantle::Expectation)
        );
        assert_eq!(
            v.default_resolution(),
            &HtmlAutocompleteDefaultResolution::NotRequired
        );
    }
}
#[test]
fn reset_default_reuses_actual_source_association_and_retains_origin() {
    let f = fixture("input", &[], &[("autocomplete", "off")]);
    let cx = context(&f, Some(HtmlFormAssociationState::ResetAgainstCapture));
    let r = report(&f, &cx);
    let v = value(&r);
    assert_eq!(setting(v), HtmlFormAutocomplete::Off);
    assert!(
        matches!(v.default_resolution(),HtmlAutocompleteDefaultResolution::Resolved{owner,..} if owner.owner()==Some(&f.form)&&owner.origin()==HtmlFormOwnerOrigin::ResetAgainstCapture)
    );
}
#[test]
fn type_control_and_consulted_form_raw_caps_are_real_terminal_admission_boundaries() {
    for (attrs, form_attrs, cap) in [
        (&[("type", "password")][..], &[][..], 7),
        (&[("autocomplete", "invalid")][..], &[][..], 6),
        (&[][..], &[("autocomplete", "invalid")][..], 6),
    ] {
        let f = fixture("input", attrs, form_attrs);
        let cx = context(
            &f,
            Some(HtmlFormAssociationState::EstablishedCurrent(Some(
                f.form.clone(),
            ))),
        );
        let mut ss = session(
            &f.capture,
            &cx,
            limits(),
            cap,
            HtmlSemanticCancellation::new(),
        );
        let failure = ss.query(&[request(&f.control)]).unwrap_err();
        assert!(matches!(
            failure.kind(),
            HtmlSemanticFailureKind::Attribute(_)
        ));
        assert_eq!(ss.query(&[]).unwrap_err(), failure);
    }
}
#[test]
fn state_only_owner_changes_are_qualified_and_do_not_reuse_effective_defaults() {
    let f = fixture("input", &[], &[("autocomplete", "off")]);
    let a = context(&f, Some(HtmlFormAssociationState::EstablishedCurrent(None)));
    let b = context(
        &f,
        Some(HtmlFormAssociationState::EstablishedCurrent(Some(
            f.form.clone(),
        ))),
    );
    let ar = report(&f, &a);
    let br = report(&f, &b);
    assert_eq!(setting(value(&ar)), HtmlFormAutocomplete::On);
    assert_eq!(setting(value(&br)), HtmlFormAutocomplete::Off);
    assert_eq!(ar.capture(), br.capture());
    assert_ne!(ar.context(), br.context());
    assert_eq!(ar.host_revision(), f.capture.host_facts_revision());
}
#[test]
fn actual_vector_work_and_output_admission_have_exact_and_one_short_boundaries() {
    let f = fixture(
        "input",
        &[("autocomplete", "section-Parent shipping home tel")],
        &[],
    );
    let cx = context(&f, None);
    let req = request(&f.control);
    let baseline = report(&f, &cx).usage();
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
                    10000
                },
            );
            let mut ss = session(&f.capture, &cx, l, 10000, HtmlSemanticCancellation::new());
            let result = ss.query(std::slice::from_ref(&req));
            if cap == exact {
                assert!(result.is_ok(), "{resource:?}: {result:?}");
            } else {
                let failure = result.unwrap_err();
                assert!(
                    matches!(failure.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r==resource),
                    "{resource:?}: {failure:?}"
                );
                assert_eq!(ss.query(&[]).unwrap_err(), failure);
            }
            assert_eq!(ss.usage().scratch_records(), 0);
        }
    }
}
#[test]
fn late_batch_stop_discards_all_results_and_keeps_original_qualification() {
    let f = fixture("input", &[("autocomplete", "name")], &[]);
    let cx = context(&f, None);
    let baseline = report(&f, &cx).usage();
    let mut ss = session(
        &f.capture,
        &cx,
        HtmlSemanticLimits::new(
            10000000,
            2,
            1000,
            0,
            1000000,
            baseline.nested_output_records(),
        ),
        10000,
        HtmlSemanticCancellation::new(),
    );
    let req = request(&f.control);
    let failure = ss.query(&[req.clone(), req]).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::NestedOutputRecords,
            ..
        }
    ));
    assert_eq!(failure.qualification().unwrap().context(), cx.identity());
    assert_eq!(
        failure.qualification().unwrap().capture(),
        f.capture.identity()
    );
    assert_eq!(ss.usage().scratch_records(), 0);
    assert_eq!(ss.query(&[]).unwrap_err(), failure);
}
#[test]
fn cancellation_and_foreign_capture_are_terminal_without_results() {
    let f = fixture("input", &[], &[]);
    let cx = context(&f, None);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut ss = session(&f.capture, &cx, limits(), 10000, cancel);
    let failure = ss.query(&[request(&f.control)]).unwrap_err();
    assert_eq!(ss.query(&[]).unwrap_err(), failure);
    let foreign = fixture("input", &[], &[]);
    let failure = session(
        &f.capture,
        &cx,
        limits(),
        10000,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(&foreign.control)])
    .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Capture(_)
    ));
}

#[test]
fn current_nonancestor_pointer_is_used_without_substituting_nearest_form() {
    let (mut b, d) = builder();
    let ancestor = element(&mut b, &d, "form", &[("autocomplete", "off")]);
    let actual = element(&mut b, &d, "form", &[("autocomplete", "on")]);
    let control = element(&mut b, &d, "input", &[]);
    b.establish_children(&ancestor, std::slice::from_ref(&control))
        .unwrap();
    b.establish_children(&actual, &[]).unwrap();
    b.establish_children(&d, &[ancestor, actual.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let f = Fixture {
        control: c.subject_id(&control).unwrap(),
        form: c.subject_id(&actual).unwrap(),
        capture: c,
    };
    let cx = context(
        &f,
        Some(HtmlFormAssociationState::EstablishedCurrent(Some(
            f.form.clone(),
        ))),
    );
    let r = report(&f, &cx);
    let v = value(&r);
    assert_eq!(setting(v), HtmlFormAutocomplete::On);
    assert!(
        matches!(v.default_resolution(),HtmlAutocompleteDefaultResolution::Resolved{owner,..} if owner.owner()==Some(&f.form))
    );
}
#[test]
fn actual_null_needs_no_ancestry_while_reset_unknown_parent_is_missing() {
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", &[("autocomplete", "off")]);
    let control = element(&mut b, &d, "input", &[]);
    let c = b.finish().unwrap();
    let f = Fixture {
        control: c.subject_id(&control).unwrap(),
        form: c.subject_id(&form).unwrap(),
        capture: c,
    };
    let cx = context(&f, Some(HtmlFormAssociationState::EstablishedCurrent(None)));
    assert_eq!(setting(value(&report(&f, &cx))), HtmlFormAutocomplete::On);
    let cx = context(&f, Some(HtmlFormAssociationState::ResetAgainstCapture));
    let r = report(&f, &cx);
    assert_eq!(
        value(&r).default_resolution(),
        &HtmlAutocompleteDefaultResolution::Missing(HtmlSemanticMissing::Parent(f.control.clone()))
    );
}
#[test]
fn exact_utf16_raw_and_ascii_only_normalization_survive_contextual_resolution() {
    let (mut b, d) = builder();
    let form = element(&mut b, &d, "form", &[("autocomplete", "off")]);
    let raw = DocumentString::from_utf16(vec![
        0x73, 0x65, 0x63, 0x74, 0x69, 0x6f, 0x6e, 0x2d, 0x41, 0xd800, 0x20, 0x4e, 0x41, 0x4d, 0x45,
    ]);
    let control = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), None, "input"),
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), None, "autocomplete"),
                raw.clone(),
            )])
            .unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let c = b.finish().unwrap();
    let f = Fixture {
        control: c.subject_id(&control).unwrap(),
        form: c.subject_id(&form).unwrap(),
        capture: c,
    };
    let cx = context(&f, None);
    let r = report(&f, &cx);
    let v = value(&r);
    assert_eq!(v.intrinsic().raw(), Some(&raw));
    let HtmlAutocompleteProcessingState::Processed(processed) = v.intrinsic().processing() else {
        panic!("detail must process")
    };
    assert_eq!(processed.field_name(), &DocumentString::from("NAME"));
    assert_eq!(
        processed.idl_value().code_units(),
        [
            0x73, 0x65, 0x63, 0x74, 0x69, 0x6f, 0x6e, 0x2d, 0x61, 0xd800, 0x20, 0x4e, 0x41, 0x4d,
            0x45
        ]
    );
    assert_eq!(
        v.default_resolution(),
        &HtmlAutocompleteDefaultResolution::NotRequired
    );
}
#[test]
fn wrong_role_foreign_namespace_and_non_elements_are_distinct() {
    let (mut b, d) = builder();
    let div = element(&mut b, &d, "div", &[("autocomplete", "name")]);
    let foreign = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::svg(), None, "input"),
            attributes(&[("autocomplete", "name")]),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let c = b.finish().unwrap();
    let cx = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]),
        0,
    )
    .unwrap();
    let r = session(
        &c,
        &cx,
        HtmlSemanticLimits::new(10000000, 3, 0, 0, 1000000, 0),
        0,
        HtmlSemanticCancellation::new(),
    )
    .query(&[
        request(&c.subject_id(&div).unwrap()),
        request(&c.subject_id(&foreign).unwrap()),
        request(&c.subject_id(&d).unwrap()),
    ])
    .unwrap();
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::ControlAutocomplete(HtmlSemanticValue::Inapplicable)
    );
    assert_eq!(
        r.results()[1].fact(),
        &HtmlSemanticFact::ControlAutocomplete(HtmlSemanticValue::Unsupported(
            HtmlSemanticUnsupported::ForeignNamespace
        ))
    );
    assert_eq!(
        r.results()[2].fact(),
        &HtmlSemanticFact::ControlAutocomplete(HtmlSemanticValue::Inapplicable)
    );
    assert_eq!(r.usage().nested_output_records(), 0);
    assert_eq!(r.usage().peak_scratch_records(), 0);
}

#[test]
fn non_ascii_whitespace_and_surrogate_invalid_raw_default_without_loss() {
    for units in [
        vec![0x6e, 0x61, 0x6d, 0x65, 0xa0],
        vec![0xd800, 0x20, 0x6e, 0x61, 0x6d, 0x65],
    ] {
        let (mut b, d) = builder();
        let form = element(&mut b, &d, "form", &[("autocomplete", "off")]);
        let raw = DocumentString::from_utf16(units);
        let control = b
            .add_element(
                &d,
                QualifiedName::new(Namespace::html(), None, "input"),
                OrderedAttributes::new([Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "autocomplete"),
                    raw.clone(),
                )])
                .unwrap(),
                HtmlCapturedIsValue::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let c = b.finish().unwrap();
        let f = Fixture {
            control: c.subject_id(&control).unwrap(),
            form: c.subject_id(&form).unwrap(),
            capture: c,
        };
        let cx = context(
            &f,
            Some(HtmlFormAssociationState::EstablishedCurrent(Some(
                f.form.clone(),
            ))),
        );
        let r = report(&f, &cx);
        let v = value(&r);
        assert_eq!(v.intrinsic().raw(), Some(&raw));
        assert!(matches!(
            v.intrinsic().syntax(),
            HtmlAutocompleteSyntaxState::Invalid(_)
        ));
        assert_eq!(setting(v), HtmlFormAutocomplete::Off);
        assert!(default(v).idl_value().is_empty());
        assert_eq!(
            v.credential_type(),
            Some(HtmlAutocompleteCredentialOutcome::NotAssigned)
        );
    }
}

#[test]
fn null_default_retains_one_admitted_payload_record_and_zero_temporary_capacity() {
    let f = fixture("input", &[], &[]);
    let cx = context(&f, Some(HtmlFormAssociationState::EstablishedCurrent(None)));
    let l = HtmlSemanticLimits::new(10000000, 1, 0, 0, 1000000, 1);
    let r = session(&f.capture, &cx, l, 0, HtmlSemanticCancellation::new())
        .query(&[request(&f.control)])
        .unwrap();
    assert_eq!(setting(value(&r)), HtmlFormAutocomplete::On);
    assert_eq!(r.usage().nested_output_records(), 1);
    assert_eq!(r.usage().peak_scratch_records(), 0);
    let mut ss = session(
        &f.capture,
        &cx,
        HtmlSemanticLimits::new(10000000, 1, 0, 0, 1000000, 0),
        0,
        HtmlSemanticCancellation::new(),
    );
    let failure = ss.query(&[request(&f.control)]).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::NestedOutputRecords,
            limit: 0,
            required: 1
        }
    ));
    assert_eq!(ss.query(&[]).unwrap_err(), failure);
}
