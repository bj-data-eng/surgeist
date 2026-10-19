use surgeist_html::*;
use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::{
    HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlCapturedIsValue,
    HtmlControlFacts, HtmlControlFactsLimits, HtmlFormAssociation, HtmlFormAssociationState,
    HtmlScriptingStatus, HtmlSemanticCancellation, HtmlSemanticContext, HtmlSemanticContextInput,
    HtmlSemanticFact, HtmlSemanticLimits, HtmlSemanticQuery, HtmlSemanticRequest,
    HtmlSemanticSession, HtmlSemanticValue,
};

fn element(local: &str) -> QualifiedName {
    QualifiedName::new(Namespace::html(), None, local)
}
fn attribute(kind: KnownHtmlAttribute, value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(kind.qualified_name(), value)
}
fn command(value: &str) -> HtmlAttributeReport<HtmlCommandReadback> {
    let raw = attribute(KnownHtmlAttribute::Command, value);
    HtmlValueInterpreter::new(1000)
        .command(&element("button"), Some(&raw))
        .unwrap()
}

// Literal expectations come from HTML's command table, not the implementation's
// canonical_keyword method or a vocabulary discovery record.
const STANDARDS: [(&str, &str, HtmlStandardCommand); 6] = [
    (
        "toggle-popover",
        "ToGgLe-PoPoVeR",
        HtmlStandardCommand::TogglePopover,
    ),
    (
        "show-popover",
        "SHOW-POPOVER",
        HtmlStandardCommand::ShowPopover,
    ),
    (
        "hide-popover",
        "HiDe-PoPoVeR",
        HtmlStandardCommand::HidePopover,
    ),
    ("close", "CLOSE", HtmlStandardCommand::Close),
    (
        "request-close",
        "ReQuEsT-ClOsE",
        HtmlStandardCommand::RequestClose,
    ),
    ("show-modal", "SHOW-MODAL", HtmlStandardCommand::ShowModal),
];

#[test]
fn all_standard_commands_match_ascii_case_and_emit_canonical_spelling() {
    for (canonical, mixed, state) in STANDARDS {
        for spelling in [canonical, mixed] {
            let report = command(spelling);
            assert_eq!(
                report.raw().unwrap().code_units(),
                spelling.encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(
                report.intrinsic().unwrap().state(),
                HtmlCommandState::Standard(state)
            );
            assert_eq!(report.intrinsic().unwrap().custom_value(), None);
            assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
            assert_eq!(
                report.recognition(),
                HtmlAttributeRecognition::Keyword {
                    matched: canonical,
                    canonical
                }
            );
            assert_eq!(
                report.context_author(),
                HtmlAttributeContextAuthorState::NotRequired
            );
        }
        let emitted = HtmlCommandKeyword::Standard(state).into_attribute();
        assert_eq!(
            emitted.name(),
            &KnownHtmlAttribute::Command.qualified_name()
        );
        assert_eq!(
            emitted.value().code_units(),
            canonical.encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(state.into_attribute(), emitted);
    }
}

#[test]
fn missing_command_is_unknown_with_missing_author_and_recognition() {
    let report = HtmlValueInterpreter::new(0)
        .command(&element("button"), None::<&surgeist_html::Attribute>)
        .unwrap();
    assert_eq!(report.observation(), HtmlAttributeObservation::Missing);
    assert_eq!(report.recognition(), HtmlAttributeRecognition::Missing);
    assert_eq!(report.author(), HtmlAttributeAuthorState::Missing);
    assert_eq!(
        report.intrinsic().unwrap().state(),
        HtmlCommandState::Unknown
    );
    assert_eq!(report.intrinsic().unwrap().custom_value(), None);
    assert_eq!(report.raw(), None);
}

#[test]
fn empty_and_unknown_commands_recover_unknown_without_repair() {
    for value in [
        "",
        "unknown",
        "toggle",
        "-",
        " show-modal",
        "show-modal ",
        " --x",
        "\t--x",
        "ſhow-modal",
        "ＣLOSE",
    ] {
        let report = command(value);
        assert_eq!(
            report.intrinsic().unwrap().state(),
            HtmlCommandState::Unknown,
            "{value:?}"
        );
        assert_eq!(report.intrinsic().unwrap().custom_value(), None);
        assert_eq!(
            report.author(),
            HtmlAttributeAuthorState::Nonconforming(
                HtmlAttributeConformanceError::UnrecognizedKeyword
            )
        );
        assert_eq!(
            report.recognition(),
            if value.is_empty() {
                HtmlAttributeRecognition::Empty
            } else {
                HtmlAttributeRecognition::Unrecognized
            }
        );
        assert_eq!(
            report.observation(),
            if value.is_empty() {
                HtmlAttributeObservation::PresentEmpty
            } else {
                HtmlAttributeObservation::PresentNonempty
            }
        );
        assert_eq!(
            report.raw().unwrap().code_units(),
            value.encode_utf16().collect::<Vec<_>>()
        );
    }
}

#[test]
fn custom_prefix_retains_bare_hyphens_case_interior_space_and_exact_utf16() {
    let values = [
        vec![0x2D, 0x2D],
        vec![0x2D, 0x2D, 0x2D],
        "--SHOW-MODAL".encode_utf16().collect(),
        "--a b".encode_utf16().collect(),
        vec![0x2D, 0x2D, 0, 0xD83D, 0xDE00],
        vec![0x2D, 0x2D, 0xD800],
        vec![0x2D, 0x2D, 0xDC00],
    ];
    for units in values {
        let raw = attribute(
            KnownHtmlAttribute::Command,
            DocumentString::from_utf16(units.clone()),
        );
        let report = HtmlValueInterpreter::new(units.len())
            .command(&element("button"), Some(&raw))
            .unwrap();
        assert_eq!(
            report.recognition(),
            HtmlAttributeRecognition::CustomCommand
        );
        assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
        assert_eq!(
            report.intrinsic().unwrap().state(),
            HtmlCommandState::Custom
        );
        assert_eq!(
            report
                .intrinsic()
                .unwrap()
                .custom_value()
                .unwrap()
                .code_units(),
            units
        );
        let native =
            HtmlCustomCommandKeyword::new(DocumentString::from_utf16(units.clone())).unwrap();
        assert_eq!(native.value().code_units(), units);
        let emitted = HtmlCommandKeyword::Custom(native).into_attribute();
        assert_eq!(emitted, raw);
    }
}

#[test]
fn custom_ascii_boundary_whitespace_rejects_authorship_but_preserves_processing() {
    for tail in [0x09, 0x0A, 0x0C, 0x0D, 0x20] {
        let units = vec![0x2D, 0x2D, tail];
        let raw = attribute(
            KnownHtmlAttribute::Command,
            DocumentString::from_utf16(units.clone()),
        );
        let report = HtmlValueInterpreter::new(3)
            .command(&element("button"), Some(&raw))
            .unwrap();
        assert_eq!(
            report.recognition(),
            HtmlAttributeRecognition::CustomCommand
        );
        assert_eq!(
            report.author(),
            HtmlAttributeAuthorState::Nonconforming(
                HtmlAttributeConformanceError::NonconformingKeyword
            )
        );
        assert_eq!(
            report.intrinsic().unwrap().state(),
            HtmlCommandState::Custom
        );
        assert_eq!(
            report
                .intrinsic()
                .unwrap()
                .custom_value()
                .unwrap()
                .code_units(),
            units
        );
        assert_eq!(
            HtmlCustomCommandKeyword::new(DocumentString::from_utf16(units)),
            Err(HtmlCustomCommandError::BoundaryWhitespace)
        );
    }
}

#[test]
fn custom_nonascii_whitespace_tails_pass_only_the_selected_boundary_check() {
    // Source-qualified Infra ASCII whitespace disposition, not Unicode trim.
    for tail in [0xA0, 0x2003, 0x2028] {
        let units = vec![0x2D, 0x2D, tail];
        let raw = attribute(
            KnownHtmlAttribute::Command,
            DocumentString::from_utf16(units.clone()),
        );
        let report = HtmlValueInterpreter::new(3)
            .command(&element("button"), Some(&raw))
            .unwrap();
        assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
        assert_eq!(
            report
                .intrinsic()
                .unwrap()
                .custom_value()
                .unwrap()
                .code_units(),
            units
        );
        assert_eq!(
            HtmlCustomCommandKeyword::new(DocumentString::from_utf16(units))
                .unwrap()
                .value(),
            raw.value()
        );
    }
}

#[test]
fn strict_custom_constructor_rejects_missing_prefix_without_normalization() {
    for value in ["", "-", "x--", " --", "\t--", "show-modal", "−−x"] {
        assert_eq!(
            HtmlCustomCommandKeyword::new(value),
            Err(HtmlCustomCommandError::MissingPrefix)
        );
    }
    for tail in [0xD800, 0xDC00] {
        let report = HtmlValueInterpreter::new(1)
            .command(
                &element("button"),
                Some(&attribute(
                    KnownHtmlAttribute::Command,
                    DocumentString::from_utf16(vec![tail]),
                )),
            )
            .unwrap();
        assert_eq!(
            report.intrinsic().unwrap().state(),
            HtmlCommandState::Unknown
        );
    }
}

#[test]
fn all_action_keywords_match_ascii_case_and_emit_their_own_domain() {
    for (literal, mixed, state) in [
        ("toggle", "ToGgLe", HtmlPopoverTargetAction::Toggle),
        ("show", "SHOW", HtmlPopoverTargetAction::Show),
        ("hide", "HiDe", HtmlPopoverTargetAction::Hide),
    ] {
        let raw = attribute(KnownHtmlAttribute::Popovertargetaction, mixed);
        let report = HtmlValueInterpreter::new(16)
            .popovertargetaction(&element("button"), Some(&raw))
            .unwrap();
        assert_eq!(report.intrinsic(), Some(&state));
        assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
        assert_eq!(
            report.recognition(),
            HtmlAttributeRecognition::Keyword {
                matched: literal,
                canonical: literal
            }
        );
        assert_eq!(report.raw_attribute(), Some(&raw));
        assert_eq!(
            state.into_attribute(),
            attribute(KnownHtmlAttribute::Popovertargetaction, literal)
        );
        assert_eq!(
            report.context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
    }
}

#[test]
fn action_missing_empty_and_unknown_keep_distinct_reports_while_defaulting_toggle() {
    let interpreter = HtmlValueInterpreter::new(64);
    let missing = interpreter
        .popovertargetaction(&element("button"), None::<&surgeist_html::Attribute>)
        .unwrap();
    assert_eq!(missing.intrinsic(), Some(&HtmlPopoverTargetAction::Toggle));
    assert_eq!(missing.recognition(), HtmlAttributeRecognition::Missing);
    assert_eq!(missing.author(), HtmlAttributeAuthorState::Missing);
    for value in [
        "",
        "unknown",
        "toggle-popover",
        " toggle",
        "show ",
        "--x",
        "ſhow",
    ] {
        let raw = attribute(KnownHtmlAttribute::Popovertargetaction, value);
        let report = interpreter
            .popovertargetaction(&element("button"), Some(&raw))
            .unwrap();
        assert_eq!(report.intrinsic(), Some(&HtmlPopoverTargetAction::Toggle));
        assert_eq!(
            report.author(),
            HtmlAttributeAuthorState::Nonconforming(
                HtmlAttributeConformanceError::UnrecognizedKeyword
            )
        );
        assert_eq!(
            report.recognition(),
            if value.is_empty() {
                HtmlAttributeRecognition::Empty
            } else {
                HtmlAttributeRecognition::Unrecognized
            }
        );
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
}

#[test]
fn all_input_states_keep_content_and_independently_check_author_presence() {
    let interpreter = HtmlValueInterpreter::new(64);
    let raw = attribute(KnownHtmlAttribute::Popovertargetaction, "SHOW");
    let input = element("input");
    for (state, allowed) in [
        (HtmlInputType::Hidden, false),
        (HtmlInputType::Text, false),
        (HtmlInputType::Search, false),
        (HtmlInputType::Telephone, false),
        (HtmlInputType::Url, false),
        (HtmlInputType::Email, false),
        (HtmlInputType::Password, false),
        (HtmlInputType::Date, false),
        (HtmlInputType::Month, false),
        (HtmlInputType::Week, false),
        (HtmlInputType::Time, false),
        (HtmlInputType::LocalDateTime, false),
        (HtmlInputType::Number, false),
        (HtmlInputType::Range, false),
        (HtmlInputType::Color, false),
        (HtmlInputType::Checkbox, false),
        (HtmlInputType::Radio, false),
        (HtmlInputType::File, false),
        (HtmlInputType::Submit, true),
        (HtmlInputType::Image, true),
        (HtmlInputType::Reset, true),
        (HtmlInputType::Button, true),
    ] {
        let report = interpreter
            .popovertargetaction_for_input(&input, Some(&raw), state)
            .unwrap();
        assert_eq!(
            report.applicability(),
            HtmlAttributeApplicability::Applicable
        );
        assert_eq!(report.intrinsic(), Some(&HtmlPopoverTargetAction::Show));
        assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
        assert_eq!(report.input_state(), Some(state));
        assert_eq!(
            report.context_author(),
            if allowed {
                HtmlAttributeContextAuthorState::Conforming
            } else {
                HtmlAttributeContextAuthorState::Nonconforming(
                    HtmlAttributeContextConformanceError::InputStateForbidsAttribute,
                )
            }
        );
        assert!(report.context_source_clause().unwrap().contains("#"));
        let missing = interpreter
            .popovertargetaction_for_input(&input, None::<&surgeist_html::Attribute>, state)
            .unwrap();
        assert_eq!(
            missing.context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
        assert_eq!(missing.author(), HtmlAttributeAuthorState::Missing);
        assert_eq!(missing.intrinsic(), Some(&HtmlPopoverTargetAction::Toggle));
    }
}

#[test]
fn unknown_input_author_context_and_invalid_content_are_independent() {
    let interpreter = HtmlValueInterpreter::new(64);
    let raw = attribute(KnownHtmlAttribute::Popovertargetaction, "nonsense");
    let report = interpreter
        .popovertargetaction(&element("input"), Some(&raw))
        .unwrap();
    assert_eq!(
        report.context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
    assert_eq!(report.intrinsic(), Some(&HtmlPopoverTargetAction::Toggle));
    for state in [HtmlInputType::Submit, HtmlInputType::Text] {
        let report = interpreter
            .popovertargetaction_for_input(&element("input"), Some(&raw), state)
            .unwrap();
        assert_eq!(
            report.author(),
            HtmlAttributeAuthorState::Nonconforming(
                HtmlAttributeConformanceError::UnrecognizedKeyword
            )
        );
        assert_eq!(report.intrinsic(), Some(&HtmlPopoverTargetAction::Toggle));
        assert_eq!(
            report.context_author(),
            if state == HtmlInputType::Submit {
                HtmlAttributeContextAuthorState::Conforming
            } else {
                HtmlAttributeContextAuthorState::Nonconforming(
                    HtmlAttributeContextConformanceError::InputStateForbidsAttribute,
                )
            }
        );
    }
}

#[test]
fn actual_type_recovery_supplies_text_context_for_missing_and_invalid_type() {
    let interpreter = HtmlValueInterpreter::new(64);
    let input = element("input");
    let action = attribute(KnownHtmlAttribute::Popovertargetaction, "show");
    let invalid = attribute(KnownHtmlAttribute::Type, "not-a-type");
    for raw in [None, Some(&invalid)] {
        let report = interpreter.input_type(&input, raw).unwrap();
        assert_eq!(report.intrinsic(), Some(&HtmlInputType::Text));
        let action = interpreter
            .popovertargetaction_for_input(&input, Some(&action), *report.intrinsic().unwrap())
            .unwrap();
        assert_eq!(
            action.context_author(),
            HtmlAttributeContextAuthorState::Nonconforming(
                HtmlAttributeContextConformanceError::InputStateForbidsAttribute
            )
        );
    }
}

#[test]
fn input_context_to_button_is_rejected_instead_of_silently_ignored() {
    let raw = attribute(KnownHtmlAttribute::Popovertargetaction, "hide");
    let report = HtmlValueInterpreter::new(4)
        .popovertargetaction_for_input(&element("button"), Some(&raw), HtmlInputType::Button)
        .unwrap();
    assert_eq!(
        report.applicability(),
        HtmlAttributeApplicability::DifferentContext
    );
    assert_eq!(
        report.context_author(),
        HtmlAttributeContextAuthorState::Unapplied
    );
    assert_eq!(report.intrinsic(), None);
    assert_eq!(report.input_state(), Some(HtmlInputType::Button));
    assert_eq!(report.raw_attribute(), Some(&raw));
}

#[test]
fn expanded_names_are_exact_and_prefixes_remain_evidence() {
    let interpreter = HtmlValueInterpreter::new(64);
    let button = QualifiedName::new(Namespace::html(), Some("h".into()), "button");
    let raw = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("p".into()), "command"),
        "--x",
    );
    let report = interpreter.command(&button, Some(&raw)).unwrap();
    assert_eq!(report.element(), &button);
    assert_eq!(report.raw_attribute(), Some(&raw));
    assert_eq!(
        report.applicability(),
        HtmlAttributeApplicability::Applicable
    );
    for local in ["BUTTON", "div", "input"] {
        let report = interpreter.command(&element(local), Some(&raw)).unwrap();
        assert_eq!(
            report.applicability(),
            HtmlAttributeApplicability::DifferentElement
        );
        assert_eq!(report.author(), HtmlAttributeAuthorState::Unapplied);
        assert_eq!(report.recognition(), HtmlAttributeRecognition::Unapplied);
        assert_eq!(report.intrinsic(), None);
    }
    for namespace in [Namespace::none(), Namespace::svg(), Namespace::mathml()] {
        let report = interpreter
            .command(&QualifiedName::new(namespace, None, "button"), Some(&raw))
            .unwrap();
        assert_eq!(
            report.applicability(),
            HtmlAttributeApplicability::ForeignElementNamespace
        );
        assert_eq!(report.intrinsic(), None);
    }
    for name in [
        QualifiedName::new(Namespace::none(), None, "COMMAND"),
        QualifiedName::new(Namespace::html(), None, "command"),
        KnownHtmlAttribute::Popovertargetaction.qualified_name(),
    ] {
        let wrong = Attribute::new(name, "--x");
        let report = interpreter.command(&button, Some(&wrong)).unwrap();
        assert_eq!(
            report.applicability(),
            HtmlAttributeApplicability::DifferentAttribute
        );
        assert_eq!(report.raw_attribute(), Some(&wrong));
        assert_eq!(report.intrinsic(), None);
    }
}

#[test]
fn action_expanded_roles_exclude_global_and_foreign_elements() {
    let raw = attribute(KnownHtmlAttribute::Popovertargetaction, "show");
    let interpreter = HtmlValueInterpreter::new(64);
    for local in ["button", "input"] {
        assert_eq!(
            interpreter
                .popovertargetaction(&element(local), Some(&raw))
                .unwrap()
                .intrinsic(),
            Some(&HtmlPopoverTargetAction::Show)
        );
    }
    for local in ["div", "BUTTON", "INPUT"] {
        assert_eq!(
            interpreter
                .popovertargetaction(&element(local), Some(&raw))
                .unwrap()
                .applicability(),
            HtmlAttributeApplicability::DifferentElement
        );
    }
    for namespace in [Namespace::none(), Namespace::svg(), Namespace::mathml()] {
        assert_eq!(
            interpreter
                .popovertargetaction(&QualifiedName::new(namespace, None, "input"), Some(&raw))
                .unwrap()
                .applicability(),
            HtmlAttributeApplicability::ForeignElementNamespace
        );
    }
    let wrong = attribute(KnownHtmlAttribute::Command, "--x");
    assert_eq!(
        interpreter
            .popovertargetaction(&element("button"), Some(&wrong))
            .unwrap()
            .applicability(),
        HtmlAttributeApplicability::DifferentAttribute
    );
}

#[test]
fn entire_raw_caps_precede_wrong_role_identity_and_context() {
    let units = vec![0x2D, 0x2D, 0xD800, 0x20, 0x78, 0xDC00];
    let command = attribute(
        KnownHtmlAttribute::Command,
        DocumentString::from_utf16(units.clone()),
    );
    let action = attribute(
        KnownHtmlAttribute::Popovertargetaction,
        DocumentString::from_utf16(units),
    );
    let exact = HtmlValueInterpreter::new(6);
    assert_eq!(
        exact
            .command(&element("button"), Some(&command))
            .unwrap()
            .input_code_units(),
        6
    );
    assert_eq!(
        exact
            .popovertargetaction(&element("input"), Some(&action))
            .unwrap()
            .input_code_units(),
        6
    );
    let short = HtmlValueInterpreter::new(5);
    for error in [
        short
            .command(&element("button"), Some(&command))
            .unwrap_err(),
        short.command(&element("div"), Some(&command)).unwrap_err(),
        short
            .command(&element("button"), Some(&action))
            .unwrap_err(),
        short
            .popovertargetaction(&element("input"), Some(&action))
            .unwrap_err(),
        short
            .popovertargetaction(&element("div"), Some(&action))
            .unwrap_err(),
        short
            .popovertargetaction(&element("button"), Some(&command))
            .unwrap_err(),
        short
            .popovertargetaction_for_input(&element("button"), Some(&action), HtmlInputType::Text)
            .unwrap_err(),
    ] {
        assert_eq!(error.capacity(), 5);
        assert_eq!(error.required(), 6);
    }
}

#[test]
fn reports_keep_exact_custom_payload_after_raw_drop_and_independent_calls() {
    let interpreter = HtmlValueInterpreter::new(8);
    let report = {
        let raw = attribute(
            KnownHtmlAttribute::Command,
            DocumentString::from_utf16(vec![0x2D, 0x2D, 0, 0xD800, 0x20]),
        );
        interpreter.command(&element("button"), Some(&raw)).unwrap()
    };
    let _other = command("close");
    assert_eq!(
        report.raw().unwrap().code_units(),
        &[0x2D, 0x2D, 0, 0xD800, 0x20]
    );
    assert_eq!(
        report
            .intrinsic()
            .unwrap()
            .custom_value()
            .unwrap()
            .code_units(),
        &[0x2D, 0x2D, 0, 0xD800, 0x20]
    );
    assert_eq!(
        report.author(),
        HtmlAttributeAuthorState::Nonconforming(
            HtmlAttributeConformanceError::NonconformingKeyword
        )
    );
}

#[test]
fn reports_bind_defining_pin_rows_and_context_sources() {
    let report = command("--x");
    assert_eq!(report.attribute(), KnownHtmlAttribute::Command);
    assert_eq!(report.inventory_rows(), &[28]);
    assert_eq!(
        report.source_clause(),
        "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-button-command"
    );
    assert_eq!(report.vocabulary_source_clause(), None);
    let raw = attribute(KnownHtmlAttribute::Popovertargetaction, "show");
    let report = HtmlValueInterpreter::new(4)
        .popovertargetaction_for_input(&element("input"), Some(&raw), HtmlInputType::Image)
        .unwrap();
    assert_eq!(report.inventory_rows(), &[121]);
    assert_eq!(
        report.source_clause(),
        "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-popovertargetaction"
    );
    assert!(
        report
            .context_source_clause()
            .unwrap()
            .ends_with("#image-button-state-(type=image)")
    );
}

#[test]
fn present_empty_or_unknown_command_still_excludes_auto_submitter_branch() {
    for (name, value) in [
        ("command", ""),
        ("command", "not-a-command"),
        ("commandfor", ""),
    ] {
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(10, 1000, 100, 100000, 10, 0, 0, 16_777_216),
            HtmlHostFactsRevision::fresh(),
        );
        let document = builder
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let button = builder
            .add_element(
                &document,
                element("button"),
                OrderedAttributes::new([
                    Attribute::new(QualifiedName::new(Namespace::none(), None, name), value),
                    attribute(KnownHtmlAttribute::Formmethod, "post"),
                ])
                .unwrap(),
                HtmlCapturedIsValue::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let capture = builder.finish().unwrap();
        let subject = capture.subject_id(&button).unwrap();
        let facts = HtmlControlFacts::new(
            &capture,
            capture.host_facts_revision().clone(),
            vec![HtmlFormAssociation::new(
                subject.clone(),
                HtmlFormAssociationState::EstablishedCurrent(None),
            )],
            vec![],
            vec![],
            vec![],
            HtmlControlFactsLimits::new(10, 100000),
        )
        .unwrap();
        let context = HtmlSemanticContext::new(
            &capture,
            capture.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]).with_controls(facts),
            0,
        )
        .unwrap();
        let mut session = HtmlSemanticSession::new(
            &capture,
            &context,
            HtmlValueInterpreter::new(1000),
            HtmlSemanticLimits::new(10000000, 10, 1000, 100, 100000, 1000),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let report = session
            .query(&[HtmlSemanticRequest::new(
                subject,
                HtmlSemanticQuery::SubmitterFormMethod,
            )])
            .unwrap();
        let HtmlSemanticFact::SubmitterFormMethod(HtmlSemanticValue::Known(method)) =
            report.results()[0].fact()
        else {
            panic!("expected actual form-method observation")
        };
        assert_eq!(
            method.intrinsic().button_submitter_context(),
            Some(HtmlButtonSubmitterContext::NotSubmitButton)
        );
    }
}

#[test]
fn every_button_type_keeps_local_command_and_action_without_target_facts() {
    let interpreter = HtmlValueInterpreter::new(64);
    let button = element("button");
    let command = attribute(KnownHtmlAttribute::Command, "close");
    let action = attribute(KnownHtmlAttribute::Popovertargetaction, "hide");
    for (raw_type, expected) in [
        (None, HtmlButtonTypeState::Auto),
        (Some("invalid"), HtmlButtonTypeState::Auto),
        (
            Some("submit"),
            HtmlButtonTypeState::Keyword(HtmlButtonType::Submit),
        ),
        (
            Some("reset"),
            HtmlButtonTypeState::Keyword(HtmlButtonType::Reset),
        ),
        (
            Some("button"),
            HtmlButtonTypeState::Keyword(HtmlButtonType::Button),
        ),
    ] {
        let raw_type = raw_type.map(|value| attribute(KnownHtmlAttribute::Type, value));
        assert_eq!(
            interpreter
                .button_type(&button, raw_type.as_ref())
                .unwrap()
                .intrinsic(),
            Some(&expected)
        );
        assert_eq!(
            interpreter
                .command(&button, Some(&command))
                .unwrap()
                .intrinsic()
                .unwrap()
                .state(),
            HtmlCommandState::Standard(HtmlStandardCommand::Close)
        );
        assert_eq!(
            interpreter
                .popovertargetaction(&button, Some(&action))
                .unwrap()
                .intrinsic(),
            Some(&HtmlPopoverTargetAction::Hide)
        );
    }
}

#[test]
fn prefixed_action_preserves_exact_units_and_unknown_input_context_after_drop() {
    let interpreter = HtmlValueInterpreter::new(3);
    let input = QualifiedName::new(Namespace::html(), Some("h".into()), "input");
    let name = QualifiedName::new(Namespace::none(), Some("p".into()), "popovertargetaction");
    let report = {
        let raw = Attribute::new(
            name.clone(),
            DocumentString::from_utf16(vec![0xD800, 0, 0xDC00]),
        );
        interpreter.popovertargetaction(&input, Some(&raw)).unwrap()
    };
    let _other = interpreter
        .popovertargetaction(&element("button"), None::<&surgeist_html::Attribute>)
        .unwrap();
    assert_eq!(report.element(), &input);
    assert_eq!(report.raw_attribute().unwrap().name(), &name);
    assert_eq!(report.raw().unwrap().code_units(), &[0xD800, 0, 0xDC00]);
    assert_eq!(report.intrinsic(), Some(&HtmlPopoverTargetAction::Toggle));
    assert_eq!(report.recognition(), HtmlAttributeRecognition::Unrecognized);
    assert_eq!(
        report.context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
    assert_eq!(
        interpreter
            .popovertargetaction(&input, None::<&surgeist_html::Attribute>)
            .unwrap()
            .context_author(),
        HtmlAttributeContextAuthorState::NotRequired
    );
}

#[test]
fn full_custom_tail_is_admitted_even_when_prefix_recognition_is_constant_work() {
    let mut units = vec![0x2D, 0x2D];
    units.extend(std::iter::repeat_n(0xD800, 2048));
    let raw = attribute(
        KnownHtmlAttribute::Command,
        DocumentString::from_utf16(units.clone()),
    );
    let report = HtmlValueInterpreter::new(2050)
        .command(&element("button"), Some(&raw))
        .unwrap();
    assert_eq!(
        report
            .intrinsic()
            .unwrap()
            .custom_value()
            .unwrap()
            .code_units(),
        units
    );
    let error = HtmlValueInterpreter::new(2049)
        .command(&element("div"), Some(&raw))
        .unwrap_err();
    assert_eq!(error.capacity(), 2049);
    assert_eq!(error.required(), 2050);
}
