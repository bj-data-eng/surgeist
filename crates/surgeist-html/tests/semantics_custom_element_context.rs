use surgeist_html::*;

fn builder() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 1000, 100, 100000, 100, 10, 100),
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
fn definition(name: &str, local: &str) -> HtmlCustomElementDefinition {
    HtmlCustomElementDefinition::new(name.into(), local.into(), false, false)
}
fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    local: &str,
    current: Option<&str>,
    saved: HtmlCapturedIsValue,
    registry: HtmlCaptureRegistryInput,
) -> HtmlCaptureBuilderSubject {
    let attrs = OrderedAttributes::new(
        current.map(|v| Attribute::new(QualifiedName::new(Namespace::none(), None, "is"), v)),
    )
    .unwrap();
    b.add_element(
        d,
        QualifiedName::new(Namespace::html(), None, local),
        attrs,
        saved,
        registry,
    )
    .unwrap()
}
fn context(c: &HtmlCapture) -> HtmlSemanticContext {
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]),
        0,
    )
    .unwrap()
}
fn limits(work: usize, bytes: usize) -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(work, 100, 100, 100, bytes, 100)
}
fn session<'a>(
    c: &'a HtmlCapture,
    cx: &'a HtmlSemanticContext,
    work: usize,
    bytes: usize,
    cap: usize,
    cancel: HtmlSemanticCancellation,
) -> HtmlSemanticSession<'a> {
    HtmlSemanticSession::new(
        c,
        cx,
        HtmlValueInterpreter::new(cap),
        limits(work, bytes),
        cancel,
    )
    .unwrap()
}
fn request(c: &HtmlCapture, e: &HtmlCaptureBuilderSubject) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(
        c.subject_id(e).unwrap(),
        HtmlSemanticQuery::IsAttributeAuthorContext,
    )
}
fn read(c: &HtmlCapture, e: &HtmlCaptureBuilderSubject) -> HtmlCustomElementIsAuthorContext {
    let cx = context(c);
    let report = session(
        c,
        &cx,
        1000000,
        1000000,
        100000,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(c, e)])
    .unwrap();
    assert_eq!(report.capture(), c.identity());
    assert_eq!(report.host_revision(), c.host_facts_revision());
    assert_eq!(report.usage().scratch_records(), 0);
    match report.results()[0].fact() {
        HtmlSemanticFact::IsAttributeAuthorContext(HtmlSemanticValue::Known(v)) => v.clone(),
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn actual_five_states_preserve_all_source_transient_associations() {
    for state in [
        HtmlCustomElementState::Undefined,
        HtmlCustomElementState::Failed,
        HtmlCustomElementState::Uncustomized,
        HtmlCustomElementState::Precustomized,
        HtmlCustomElementState::Custom,
    ] {
        for association in 0..3 {
            let (mut b, d) = builder();
            let r = b
                .add_registry(Some(vec![definition("x-host", "x-host")]))
                .unwrap();
            let handle = b.definition_handle(&r, 0).unwrap();
            let e = element(
                &mut b,
                &d,
                "x-host",
                Some("x-other"),
                HtmlCapturedIsValue::Value("x-other".into()),
                HtmlCaptureRegistryInput::Registry(r),
            );
            let supplied = match association {
                0 => HtmlCaptureDefinitionInput::Null,
                1 => HtmlCaptureDefinitionInput::Definition(handle.clone()),
                _ => HtmlCaptureDefinitionInput::Unavailable,
            };
            let result = b.establish_custom_element_context(
                &e,
                HtmlCapturedCustomElementState::Known(state),
                supplied,
            );
            if state == HtmlCustomElementState::Custom && association == 0 {
                assert_eq!(
                    result,
                    Err(HtmlCaptureError::CustomElementDefinitionRequired)
                );
                b.establish_custom_element_context(
                    &e,
                    HtmlCapturedCustomElementState::Known(HtmlCustomElementState::Failed),
                    HtmlCaptureDefinitionInput::Null,
                )
                .unwrap();
                continue;
            }
            result.unwrap();
            let c = b.finish().unwrap();
            let result = read(&c, &e);
            if state != HtmlCustomElementState::Custom {
                assert_eq!(
                    result.autonomous_requirement(),
                    &HtmlIsAutonomousRequirement::Allowed
                );
            } else if association == 1 {
                assert_eq!(
                    result.autonomous_requirement(),
                    &HtmlIsAutonomousRequirement::Forbidden(c.definition_id(&handle).unwrap())
                );
            } else {
                assert!(matches!(
                    result.autonomous_requirement(),
                    HtmlIsAutonomousRequirement::Unavailable(
                        HtmlIsContextUnavailable::AssociatedDefinition(_)
                    )
                ));
            }
            assert!(matches!(
                result.definition_requirement(),
                HtmlIsDefinitionRequirement::NoMatchingCustomizedDefinition { selected: Some(_) }
            ));
        }
    }
}

#[test]
fn cec1_autonomous_upgrade_retains_saved_value_and_current_presence_is_independent() {
    let (mut b, d) = builder();
    let r = b.add_registry(None).unwrap();
    let def = b
        .add_definition(&r, definition("x-host", "x-host"))
        .unwrap();
    let mut es = vec![];
    for current in [Some("x-other"), Some(""), None] {
        let e = element(
            &mut b,
            &d,
            "x-host",
            current,
            HtmlCapturedIsValue::Value("x-other".into()),
            HtmlCaptureRegistryInput::Registry(r.clone()),
        );
        b.establish_custom_element_context(
            &e,
            HtmlCapturedCustomElementState::Known(HtmlCustomElementState::Custom),
            HtmlCaptureDefinitionInput::Definition(def.clone()),
        )
        .unwrap();
        es.push(e);
    }
    let c = b.finish().unwrap();
    let valid = read(&c, &es[0]);
    assert_eq!(valid.local().author(), HtmlAttributeAuthorState::Conforming);
    assert!(matches!(
        valid.definition_requirement(),
        HtmlIsDefinitionRequirement::Unavailable(HtmlIsContextUnavailable::Definitions(_))
    ));
    assert_eq!(
        valid.autonomous_requirement(),
        &HtmlIsAutonomousRequirement::Forbidden(c.definition_id(&def).unwrap())
    );
    let invalid = read(&c, &es[1]);
    assert_eq!(
        invalid.definition_requirement(),
        &HtmlIsDefinitionRequirement::BlockedByLocalSyntax
    );
    assert_eq!(
        invalid.autonomous_requirement(),
        valid.autonomous_requirement()
    );
    assert!(matches!(
        invalid.local().author(),
        HtmlAttributeAuthorState::Nonconforming(_)
    ));
    let absent = read(&c, &es[2]);
    assert_eq!(absent.local().author(), HtmlAttributeAuthorState::Missing);
    assert_eq!(
        absent.autonomous_requirement(),
        &HtmlIsAutonomousRequirement::NotRequired
    );
    assert_eq!(
        absent.definition_requirement(),
        &HtmlIsDefinitionRequirement::NotRequired
    );
    let HtmlCapturedNodeKind::Element(element) =
        c.node(&c.subject_id(&es[2]).unwrap()).unwrap().kind()
    else {
        panic!()
    };
    assert_eq!(
        element.is_value(),
        &HtmlCapturedIsValue::Value("x-other".into())
    );
}

#[test]
fn current_content_lookup_does_not_use_saved_runtime_is_or_associated_definition() {
    let (mut b, d) = builder();
    let r = b
        .add_registry(Some(vec![
            definition("x-old", "button"),
            definition("x-new", "button"),
        ]))
        .unwrap();
    let old = b.definition_handle(&r, 0).unwrap();
    let new = b.definition_handle(&r, 1).unwrap();
    let e = element(
        &mut b,
        &d,
        "button",
        Some("x-new"),
        HtmlCapturedIsValue::Value("x-old".into()),
        HtmlCaptureRegistryInput::Registry(r),
    );
    b.establish_custom_element_context(
        &e,
        HtmlCapturedCustomElementState::Known(HtmlCustomElementState::Custom),
        HtmlCaptureDefinitionInput::Definition(old),
    )
    .unwrap();
    let c = b.finish().unwrap();
    let result = read(&c, &e);
    assert_eq!(
        result.definition_requirement(),
        &HtmlIsDefinitionRequirement::Matched(c.definition_id(&new).unwrap())
    );
    assert_eq!(
        result.autonomous_requirement(),
        &HtmlIsAutonomousRequirement::Allowed
    );
}

#[test]
fn adoption_can_separate_lookup_registry_from_associated_definition_registry() {
    let (mut b, d) = builder();
    let original = b
        .add_registry(Some(vec![definition("x-old", "button")]))
        .unwrap();
    let adopted = b
        .add_registry(Some(vec![definition("x-new", "button")]))
        .unwrap();
    let old = b.definition_handle(&original, 0).unwrap();
    let new = b.definition_handle(&adopted, 0).unwrap();
    let e = element(
        &mut b,
        &d,
        "button",
        Some("x-new"),
        HtmlCapturedIsValue::Value("x-old".into()),
        HtmlCaptureRegistryInput::Registry(adopted),
    );
    b.establish_custom_element_context(
        &e,
        HtmlCapturedCustomElementState::Known(HtmlCustomElementState::Custom),
        HtmlCaptureDefinitionInput::Definition(old),
    )
    .unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        read(&c, &e).definition_requirement(),
        &HtmlIsDefinitionRequirement::Matched(c.definition_id(&new).unwrap())
    );
}

#[test]
fn adopted_registry_never_falls_back_to_the_old_association() {
    let (mut b, d) = builder();
    let r1 = b
        .add_registry(Some(vec![definition("x-old", "button")]))
        .unwrap();
    let r2 = b.add_registry(Some(vec![])).unwrap();
    let def = b.definition_handle(&r1, 0).unwrap();
    let e = element(
        &mut b,
        &d,
        "button",
        Some("x-old"),
        HtmlCapturedIsValue::Value("x-old".into()),
        HtmlCaptureRegistryInput::Registry(r2),
    );
    b.establish_custom_element_context(
        &e,
        HtmlCapturedCustomElementState::Known(HtmlCustomElementState::Custom),
        HtmlCaptureDefinitionInput::Definition(def),
    )
    .unwrap();
    let c = b.finish().unwrap();
    let result = read(&c, &e);
    assert_eq!(
        result.definition_requirement(),
        &HtmlIsDefinitionRequirement::NoMatchingCustomizedDefinition { selected: None }
    );
    assert_eq!(
        result.autonomous_requirement(),
        &HtmlIsAutonomousRequirement::Allowed
    );
}

#[test]
fn autonomous_prohibition_survives_unavailable_current_registry_after_adoption() {
    let (mut b, d) = builder();
    let r = b
        .add_registry(Some(vec![definition("x-host", "x-host")]))
        .unwrap();
    let def = b.definition_handle(&r, 0).unwrap();
    let e = element(
        &mut b,
        &d,
        "x-host",
        Some("x-other"),
        HtmlCapturedIsValue::Value("x-other".into()),
        HtmlCaptureRegistryInput::Unavailable,
    );
    b.establish_custom_element_context(
        &e,
        HtmlCapturedCustomElementState::Known(HtmlCustomElementState::Custom),
        HtmlCaptureDefinitionInput::Definition(def.clone()),
    )
    .unwrap();
    let c = b.finish().unwrap();
    let result = read(&c, &e);
    assert_eq!(
        result.definition_requirement(),
        &HtmlIsDefinitionRequirement::Unavailable(HtmlIsContextUnavailable::Registry(
            c.subject_id(&e).unwrap()
        ))
    );
    assert_eq!(
        result.autonomous_requirement(),
        &HtmlIsAutonomousRequirement::Forbidden(c.definition_id(&def).unwrap())
    );
}

#[test]
fn definition_match_survives_unavailable_state_and_association() {
    for state in [
        HtmlCapturedCustomElementState::Unavailable,
        HtmlCapturedCustomElementState::Known(HtmlCustomElementState::Custom),
    ] {
        let (mut b, d) = builder();
        let r = b
            .add_registry(Some(vec![definition("x-good", "button")]))
            .unwrap();
        let handle = b.definition_handle(&r, 0).unwrap();
        let e = element(
            &mut b,
            &d,
            "button",
            Some("x-good"),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Registry(r),
        );
        b.establish_custom_element_context(&e, state, HtmlCaptureDefinitionInput::Unavailable)
            .unwrap();
        let c = b.finish().unwrap();
        let result = read(&c, &e);
        assert_eq!(
            result.definition_requirement(),
            &HtmlIsDefinitionRequirement::Matched(c.definition_id(&handle).unwrap())
        );
        if state == HtmlCapturedCustomElementState::Unavailable {
            assert!(matches!(
                result.autonomous_requirement(),
                HtmlIsAutonomousRequirement::Unavailable(
                    HtmlIsContextUnavailable::CustomElementState(_)
                )
            ));
        } else {
            assert!(matches!(
                result.autonomous_requirement(),
                HtmlIsAutonomousRequirement::Unavailable(
                    HtmlIsContextUnavailable::AssociatedDefinition(_)
                )
            ));
        }
        assert_eq!(
            result.local().context_author(),
            HtmlAttributeContextAuthorState::Unchecked
        );
    }
}

#[test]
fn known_noncustom_state_does_not_infer_customness_from_hyphen_or_registry() {
    let (mut b, d) = builder();
    let r = b
        .add_registry(Some(vec![definition("x-host", "x-host")]))
        .unwrap();
    let e = element(
        &mut b,
        &d,
        "x-host",
        Some("x-other"),
        HtmlCapturedIsValue::Absent,
        HtmlCaptureRegistryInput::Registry(r),
    );
    b.establish_custom_element_context(
        &e,
        HtmlCapturedCustomElementState::Known(HtmlCustomElementState::Undefined),
        HtmlCaptureDefinitionInput::Null,
    )
    .unwrap();
    let c = b.finish().unwrap();
    let result = read(&c, &e);
    assert_eq!(
        result.autonomous_requirement(),
        &HtmlIsAutonomousRequirement::Allowed
    );
    assert!(matches!(
        result.definition_requirement(),
        HtmlIsDefinitionRequirement::NoMatchingCustomizedDefinition { selected: Some(_) }
    ));
}

#[test]
fn autonomous_first_lookup_precedes_current_is_match() {
    let (mut b, d) = builder();
    let r = b
        .add_registry(Some(vec![
            definition("x-host", "x-host"),
            definition("x-other", "x-host"),
        ]))
        .unwrap();
    let autonomous = b.definition_handle(&r, 0).unwrap();
    let e = element(
        &mut b,
        &d,
        "x-host",
        Some("x-other"),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Registry(r),
    );
    let c = b.finish().unwrap();
    assert_eq!(
        read(&c, &e).definition_requirement(),
        &HtmlIsDefinitionRequirement::NoMatchingCustomizedDefinition {
            selected: Some(c.definition_id(&autonomous).unwrap())
        }
    );
}

#[test]
fn absent_invalid_null_empty_and_unknown_registry_are_distinct() {
    let (mut b, d) = builder();
    let empty = b.add_registry(Some(vec![])).unwrap();
    let partial = b.add_registry(None).unwrap();
    let none = element(
        &mut b,
        &d,
        "button",
        None,
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Unavailable,
    );
    let bad = element(
        &mut b,
        &d,
        "button",
        Some(" X-bad "),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Unavailable,
    );
    let null = element(
        &mut b,
        &d,
        "button",
        Some("x-good"),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    );
    let zero = element(
        &mut b,
        &d,
        "button",
        Some("x-good"),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Registry(empty),
    );
    let unknown = element(
        &mut b,
        &d,
        "button",
        Some("x-good"),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Unavailable,
    );
    let incomplete = element(
        &mut b,
        &d,
        "button",
        Some("x-good"),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Registry(partial),
    );
    let c = b.finish().unwrap();
    assert_eq!(
        read(&c, &none).definition_requirement(),
        &HtmlIsDefinitionRequirement::NotRequired
    );
    assert_eq!(
        read(&c, &bad).definition_requirement(),
        &HtmlIsDefinitionRequirement::BlockedByLocalSyntax
    );
    for e in [&null, &zero] {
        assert_eq!(
            read(&c, e).definition_requirement(),
            &HtmlIsDefinitionRequirement::NoMatchingCustomizedDefinition { selected: None }
        );
    }
    assert!(matches!(
        read(&c, &unknown).definition_requirement(),
        HtmlIsDefinitionRequirement::Unavailable(HtmlIsContextUnavailable::Registry(_))
    ));
    assert!(matches!(
        read(&c, &incomplete).definition_requirement(),
        HtmlIsDefinitionRequirement::Unavailable(HtmlIsContextUnavailable::Definitions(_))
    ));
}

#[test]
fn partial_registry_retains_real_records_but_does_not_claim_complete_lookup() {
    let (mut b, d) = builder();
    let r = b.add_registry(None).unwrap();
    let handle = b
        .add_definition(&r, definition("x-host", "x-host"))
        .unwrap();
    assert_eq!(b.usage().definitions(), 1);
    let before = b.usage();
    assert_eq!(
        b.add_definition(&r, definition("x-host", "button")),
        Err(HtmlCaptureError::DuplicateDefinitionName)
    );
    assert_eq!(
        b.add_definition(&r, definition("X-bad", "button")),
        Err(HtmlCaptureError::InvalidDefinitionName)
    );
    assert_eq!(b.usage(), before);
    assert_eq!(
        b.definition_handle(&r, 1),
        Err(HtmlCaptureError::UnknownDefinition)
    );
    let e = element(
        &mut b,
        &d,
        "x-host",
        Some("x-good"),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Registry(r.clone()),
    );
    b.establish_custom_element_context(
        &e,
        HtmlCapturedCustomElementState::Known(HtmlCustomElementState::Custom),
        HtmlCaptureDefinitionInput::Definition(handle.clone()),
    )
    .unwrap();
    let c = b.finish().unwrap();
    let id = c.definition_id(&handle).unwrap();
    assert_eq!(id.registry(), &c.registry_id(&r).unwrap());
    assert_eq!(
        c.definition(&id).unwrap().name(),
        &DocumentString::from("x-host")
    );
    assert!(matches!(
        c.definitions(id.registry()).unwrap(),
        HtmlCapturedDefinitions::Unavailable
    ));
    assert!(matches!(
        c.lookup_definition(
            &HtmlCapturedRegistry::Registry(id.registry().clone()),
            &Namespace::html(),
            &"x-host".into(),
            None
        ),
        Err(HtmlCaptureError::UnavailableDefinitions(_))
    ));
    assert!(matches!(
        read(&c, &e).autonomous_requirement(),
        HtmlIsAutonomousRequirement::Forbidden(_)
    ));
}

#[test]
fn complete_and_foreign_registry_handles_cannot_be_appended_or_joined() {
    let (mut b, d) = builder();
    let complete = b
        .add_registry(Some(vec![definition("x-old", "button")]))
        .unwrap();
    assert_eq!(
        b.add_definition(&complete, definition("x-new", "button")),
        Err(HtmlCaptureError::CompleteRegistry)
    );
    let (mut other, _) = builder();
    let foreign = other.add_registry(None).unwrap();
    let foreign_def = other
        .add_definition(&foreign, definition("x-foreign", "button"))
        .unwrap();
    assert_eq!(
        b.definition_handle(&foreign, 0),
        Err(HtmlCaptureError::ForeignBuilder)
    );
    let e = element(
        &mut b,
        &d,
        "button",
        None,
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    );
    assert_eq!(
        b.establish_custom_element_context(
            &e,
            HtmlCapturedCustomElementState::Unavailable,
            HtmlCaptureDefinitionInput::Definition(foreign_def.clone())
        ),
        Err(HtmlCaptureError::ForeignBuilder)
    );
    let c = b.finish().unwrap();
    assert_eq!(
        c.definition_id(&foreign_def),
        Err(HtmlCaptureError::ForeignCapture)
    );
    let other = other.finish().unwrap();
    assert_eq!(
        c.definition(&other.definition_id(&foreign_def).unwrap()),
        Err(HtmlCaptureError::ForeignCapture)
    );
}

#[test]
fn customized_custom_state_checks_saved_name_but_not_current_content() {
    for saved in [
        HtmlCapturedIsValue::Absent,
        HtmlCapturedIsValue::Value("x-other".into()),
    ] {
        let (mut b, d) = builder();
        let r = b
            .add_registry(Some(vec![definition("x-old", "button")]))
            .unwrap();
        let def = b.definition_handle(&r, 0).unwrap();
        let e = element(
            &mut b,
            &d,
            "button",
            Some("x-old"),
            saved,
            HtmlCaptureRegistryInput::Null,
        );
        let before = b.usage();
        assert_eq!(
            b.establish_custom_element_context(
                &e,
                HtmlCapturedCustomElementState::Known(HtmlCustomElementState::Custom),
                HtmlCaptureDefinitionInput::Definition(def.clone())
            ),
            Err(HtmlCaptureError::CustomElementSavedIsMismatch)
        );
        assert_eq!(b.usage(), before);
        b.establish_custom_element_context(
            &e,
            HtmlCapturedCustomElementState::Known(HtmlCustomElementState::Precustomized),
            HtmlCaptureDefinitionInput::Definition(def),
        )
        .unwrap();
    }
    let (mut b, d) = builder();
    let r = b
        .add_registry(Some(vec![definition("x-old", "button")]))
        .unwrap();
    let def = b.definition_handle(&r, 0).unwrap();
    let e = element(
        &mut b,
        &d,
        "button",
        Some("x-new"),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    );
    b.establish_custom_element_context(
        &e,
        HtmlCapturedCustomElementState::Known(HtmlCustomElementState::Custom),
        HtmlCaptureDefinitionInput::Definition(def),
    )
    .unwrap();
}

#[test]
fn custom_join_rejects_proven_namespace_and_local_mismatches_without_mutation() {
    for (namespace, local, expected) in [
        (
            Namespace::none(),
            "x-host",
            HtmlCaptureError::CustomElementDefinitionNamespaceMismatch,
        ),
        (
            Namespace::html(),
            "x-other",
            HtmlCaptureError::CustomElementDefinitionNameMismatch,
        ),
    ] {
        let (mut b, d) = builder();
        let r = b
            .add_registry(Some(vec![definition("x-host", "x-host")]))
            .unwrap();
        let def = b.definition_handle(&r, 0).unwrap();
        let e = b
            .add_element(
                &d,
                QualifiedName::new(namespace, None, local),
                OrderedAttributes::new([]).unwrap(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let before = b.usage();
        assert_eq!(
            b.establish_custom_element_context(
                &e,
                HtmlCapturedCustomElementState::Known(HtmlCustomElementState::Custom),
                HtmlCaptureDefinitionInput::Definition(def)
            ),
            Err(expected)
        );
        assert_eq!(b.usage(), before);
        b.establish_custom_element_context(
            &e,
            HtmlCapturedCustomElementState::Known(HtmlCustomElementState::Failed),
            HtmlCaptureDefinitionInput::Null,
        )
        .unwrap();
        assert_eq!(
            b.establish_custom_element_context(
                &e,
                HtmlCapturedCustomElementState::Unavailable,
                HtmlCaptureDefinitionInput::Unavailable
            ),
            Err(HtmlCaptureError::CustomElementContextAlreadyEstablished)
        );
    }
}

#[test]
fn state_unavailable_retains_each_association_without_inventing_customness() {
    for association in 0..3 {
        let (mut b, d) = builder();
        let r = b
            .add_registry(Some(vec![definition("x-host", "x-host")]))
            .unwrap();
        let def = b.definition_handle(&r, 0).unwrap();
        let e = element(
            &mut b,
            &d,
            "x-host",
            Some("x-other"),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        );
        let supplied = match association {
            0 => HtmlCaptureDefinitionInput::Null,
            1 => HtmlCaptureDefinitionInput::Definition(def),
            _ => HtmlCaptureDefinitionInput::Unavailable,
        };
        b.establish_custom_element_context(
            &e,
            HtmlCapturedCustomElementState::Unavailable,
            supplied,
        )
        .unwrap();
        let c = b.finish().unwrap();
        assert!(matches!(
            read(&c, &e).autonomous_requirement(),
            HtmlIsAutonomousRequirement::Unavailable(HtmlIsContextUnavailable::CustomElementState(
                _
            ))
        ));
    }
}

#[test]
fn raw_admission_precedes_foreign_role_and_exact_expanded_names_select_content() {
    let (mut b, d) = builder();
    let foreign = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::none(), None, "button"),
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), Some("p".into()), "is"),
                "x-long",
            )])
            .unwrap(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let ignored = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), None, "button"),
            OrderedAttributes::new([
                Attribute::new(QualifiedName::new(Namespace::none(), None, "IS"), "long"),
                Attribute::new(
                    QualifiedName::new(Namespace::from_uri("urn:x"), None, "is"),
                    "long",
                ),
            ])
            .unwrap(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let exact = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), Some("h".into()), "button"),
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), Some("p".into()), "is"),
                "x-y",
            )])
            .unwrap(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let mut s = session(&c, &cx, 100000, 100000, 5, HtmlSemanticCancellation::new());
    assert!(matches!(
        s.query(&[request(&c, &foreign)]).unwrap_err().kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    assert_eq!(
        read(&c, &ignored).local().author(),
        HtmlAttributeAuthorState::Missing
    );
    let result = read(&c, &exact);
    assert_eq!(
        result.local().raw_attribute().unwrap().name().prefix(),
        Some(&DocumentString::from("p"))
    );
    assert_eq!(
        result.local().author(),
        HtmlAttributeAuthorState::Conforming
    );
    let report = session(&c, &cx, 100000, 100000, 6, HtmlSemanticCancellation::new())
        .query(&[request(&c, &foreign)])
        .unwrap();
    assert_eq!(
        report.results()[0].fact(),
        &HtmlSemanticFact::IsAttributeAuthorContext(HtmlSemanticValue::Inapplicable)
    );
}

#[test]
fn non_element_queries_are_inapplicable_without_registry_facts() {
    let (b, d) = builder();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let report = session(&c, &cx, 100000, 100000, 0, HtmlSemanticCancellation::new())
        .query(&[request(&c, &d)])
        .unwrap();
    assert_eq!(
        report.results()[0].fact(),
        &HtmlSemanticFact::IsAttributeAuthorContext(HtmlSemanticValue::Inapplicable)
    );
}

#[test]
fn finite_work_one_short_is_sticky_qualified_and_discards_the_entire_batch() {
    let (mut b, d) = builder();
    let r = b
        .add_registry(Some(vec![definition("x-good", "button")]))
        .unwrap();
    let e = element(
        &mut b,
        &d,
        "button",
        Some("x-good"),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Registry(r),
    );
    let c = b.finish().unwrap();
    let cx = context(&c);
    let requests = [request(&c, &d), request(&c, &e)];
    let good = session(
        &c,
        &cx,
        100000,
        100000,
        100,
        HtmlSemanticCancellation::new(),
    )
    .query(&requests)
    .unwrap();
    let mut limited = session(
        &c,
        &cx,
        good.usage().work() - 1,
        100000,
        100,
        HtmlSemanticCancellation::new(),
    );
    let first = limited.query(&requests).unwrap_err();
    assert!(matches!(
        first.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            ..
        }
    ));
    assert_eq!(first.qualification().unwrap().capture(), c.identity());
    assert_eq!(first.qualification().unwrap().context(), cx.identity());
    assert_eq!(first.usage().scratch_records(), 0);
    assert_eq!(limited.query(&[]).unwrap_err(), first);
    let exact = session(
        &c,
        &cx,
        good.usage().work(),
        100000,
        100,
        HtmlSemanticCancellation::new(),
    )
    .query(&requests)
    .unwrap();
    assert_eq!(exact.results().len(), 2);
}

#[test]
fn actual_output_layout_is_admitted_before_query_and_cancellation_remains_sticky() {
    let (mut b, d) = builder();
    let e = element(
        &mut b,
        &d,
        "button",
        Some("x-good"),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    );
    let c = b.finish().unwrap();
    let cx = context(&c);
    let mut s = session(
        &c,
        &cx,
        100000,
        std::mem::size_of::<HtmlSemanticResult>() - 1,
        100,
        HtmlSemanticCancellation::new(),
    );
    let failure = s.query(&[request(&c, &e)]).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::AllocatedBytes,
            ..
        }
    ));
    assert_eq!(failure.usage().scratch_records(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), failure);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut s = session(&c, &cx, 100000, 100000, 100, cancel);
    let failure = s.query(&[request(&c, &e)]).unwrap_err();
    assert_eq!(failure.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(s.query(&[]).unwrap_err(), failure);
}

#[test]
fn partial_definition_count_limit_rejection_is_atomic() {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(10, 10, 10, 1000, 10, 1, 1),
        HtmlHostFactsRevision::fresh(),
    );
    let r = b.add_registry(None).unwrap();
    b.add_definition(&r, definition("x-first", "button"))
        .unwrap();
    let usage = b.usage();
    assert_eq!(
        b.add_definition(&r, definition("x-second", "button")),
        Err(HtmlCaptureError::LimitExceeded {
            resource: HtmlCaptureResource::Definitions,
            limit: 1,
            required: 2
        })
    );
    assert_eq!(b.usage(), usage);
}

#[test]
fn partial_record_utf16_retention_limit_is_atomic() {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(10, 10, 10, 11, 10, 1, 5),
        HtmlHostFactsRevision::fresh(),
    );
    let r = b.add_registry(None).unwrap();
    let before = b.usage();
    assert_eq!(
        b.add_definition(&r, definition("x-good", "button")),
        Err(HtmlCaptureError::LimitExceeded {
            resource: HtmlCaptureResource::Utf16Units,
            limit: 11,
            required: 12
        })
    );
    assert_eq!(b.usage(), before);
    let handle = b.add_definition(&r, definition("x-a", "button")).unwrap();
    assert_eq!(b.usage().utf16_units(), 9);
    assert_eq!(b.usage().definitions(), 1);
    assert_eq!(b.definition_handle(&r, 0).unwrap(), handle);
}

#[test]
fn raw_surrogates_and_reserved_names_keep_exact_local_failures_with_known_prohibition() {
    let (mut b, d) = builder();
    let r = b
        .add_registry(Some(vec![definition("x-host", "x-host")]))
        .unwrap();
    let def = b.definition_handle(&r, 0).unwrap();
    let values = [
        DocumentString::from_utf16(vec![120, 45, 0xd800]),
        DocumentString::from("font-face"),
        DocumentString::from("x-UPPER"),
    ];
    let mut es = vec![];
    for value in &values {
        let e = b
            .add_element(
                &d,
                QualifiedName::new(Namespace::html(), None, "x-host"),
                OrderedAttributes::new([Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "is"),
                    value.clone(),
                )])
                .unwrap(),
                HtmlCapturedIsValue::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        b.establish_custom_element_context(
            &e,
            HtmlCapturedCustomElementState::Known(HtmlCustomElementState::Custom),
            HtmlCaptureDefinitionInput::Definition(def.clone()),
        )
        .unwrap();
        es.push(e);
    }
    let c = b.finish().unwrap();
    for (index, e) in es.iter().enumerate() {
        let result = read(&c, e);
        assert_eq!(result.local().raw(), Some(&values[index]));
        assert_eq!(
            result.local().context_author(),
            HtmlAttributeContextAuthorState::Unchecked
        );
        assert!(matches!(
            result.autonomous_requirement(),
            HtmlIsAutonomousRequirement::Forbidden(_)
        ));
        if index == 0 {
            assert_eq!(
                result.local().author(),
                HtmlAttributeAuthorState::Conforming
            );
        } else {
            assert_eq!(
                result.definition_requirement(),
                &HtmlIsDefinitionRequirement::BlockedByLocalSyntax
            );
        }
    }
}

#[test]
fn every_short_work_boundary_stops_within_the_limit_and_releases_scratch() {
    let (mut b, d) = builder();
    let r = b
        .add_registry(Some(vec![
            definition("x-other", "button"),
            definition("x-good", "button"),
        ]))
        .unwrap();
    let e = element(
        &mut b,
        &d,
        "button",
        Some("x-good"),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Registry(r),
    );
    let c = b.finish().unwrap();
    let cx = context(&c);
    let requests = [request(&c, &e)];
    let complete = session(
        &c,
        &cx,
        100000,
        100000,
        100,
        HtmlSemanticCancellation::new(),
    )
    .query(&requests)
    .unwrap();
    for work in 0..complete.usage().work() {
        let mut s = session(&c, &cx, work, 100000, 100, HtmlSemanticCancellation::new());
        let failure = s.query(&requests).unwrap_err();
        assert!(matches!(
            failure.kind(),
            HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::Work,
                ..
            }
        ));
        assert!(failure.usage().work() <= work);
        assert_eq!(failure.usage().scratch_records(), 0);
        assert_eq!(s.query(&[]).unwrap_err(), failure);
    }
}

#[test]
fn foreign_request_and_changed_context_keys_never_reuse_a_retained_observation() {
    let (mut b, d) = builder();
    let e = element(
        &mut b,
        &d,
        "button",
        Some("x-good"),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    );
    let c = b.finish().unwrap();
    let cx = context(&c);
    let changed = context(&c);
    let first = session(
        &c,
        &cx,
        100000,
        100000,
        100,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(&c, &e)])
    .unwrap();
    let second = session(
        &c,
        &changed,
        100000,
        100000,
        100,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(&c, &e)])
    .unwrap();
    assert_ne!(first.context(), second.context());
    let (other, doc) = builder();
    let other = other.finish().unwrap();
    let mut s = session(
        &c,
        &cx,
        100000,
        100000,
        100,
        HtmlSemanticCancellation::new(),
    );
    let failure = s
        .query(&[request(&c, &e), request(&other, &doc)])
        .unwrap_err();
    assert_eq!(
        failure.kind(),
        &HtmlSemanticFailureKind::Capture(HtmlCaptureError::ForeignCapture)
    );
    assert_eq!(failure.qualification().unwrap().capture(), first.capture());
    assert_eq!(failure.qualification().unwrap().context(), first.context());
    assert_eq!(s.query(&[]).unwrap_err(), failure);
}

#[test]
fn association_relationship_limit_rejects_before_context_establishment() {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(10, 1, 10, 1000, 10, 1, 1),
        HtmlHostFactsRevision::fresh(),
    );
    let r = b
        .add_registry(Some(vec![definition("x-host", "x-host")]))
        .unwrap();
    let def = b.definition_handle(&r, 0).unwrap();
    let d = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let e = element(
        &mut b,
        &d,
        "x-host",
        None,
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    );
    let before = b.usage();
    assert_eq!(
        b.establish_custom_element_context(
            &e,
            HtmlCapturedCustomElementState::Known(HtmlCustomElementState::Custom),
            HtmlCaptureDefinitionInput::Definition(def)
        ),
        Err(HtmlCaptureError::LimitExceeded {
            resource: HtmlCaptureResource::Relationships,
            limit: 1,
            required: 2
        })
    );
    assert_eq!(b.usage(), before);
    b.establish_custom_element_context(
        &e,
        HtmlCapturedCustomElementState::Known(HtmlCustomElementState::Failed),
        HtmlCaptureDefinitionInput::Null,
    )
    .unwrap();
}

#[test]
fn wrong_subject_kind_rejects_context_without_establishing_a_record() {
    let (mut b, d) = builder();
    let before = b.usage();
    assert_eq!(
        b.establish_custom_element_context(
            &d,
            HtmlCapturedCustomElementState::Unavailable,
            HtmlCaptureDefinitionInput::Unavailable
        ),
        Err(HtmlCaptureError::WrongKind {
            expected: HtmlCaptureSubjectKind::Element,
            actual: HtmlCaptureSubjectKind::Document
        })
    );
    assert_eq!(b.usage(), before);
}
