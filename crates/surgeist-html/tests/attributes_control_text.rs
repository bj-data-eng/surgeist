use surgeist_html::*;

const BASE: &str =
    "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#";

fn input() -> QualifiedName {
    KnownHtmlElement::Input.qualified_name()
}
fn textarea() -> QualifiedName {
    KnownHtmlElement::Textarea.qualified_name()
}
fn dirname(value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(KnownHtmlAttribute::Dirname.qualified_name(), value)
}
fn placeholder(value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(KnownHtmlAttribute::Placeholder.qualified_name(), value)
}
fn forbidden() -> HtmlAttributeContextAuthorState {
    HtmlAttributeContextAuthorState::Nonconforming(
        HtmlAttributeContextConformanceError::InputStateForbidsAttribute,
    )
}
fn newline_error(offset: usize) -> HtmlAttributeConformanceError {
    HtmlAttributeConformanceError::InputPlaceholderContainsNewline {
        code_unit_offset: offset,
    }
}

#[test]
fn dirname_absence_and_empty_have_no_default_name_and_distinct_author_evidence() {
    let interpreter = HtmlValueInterpreter::new(0);
    for element in [input(), textarea()] {
        let absent = interpreter.dirname(&element, None).unwrap();
        assert_eq!(
            absent.applicability(),
            HtmlAttributeApplicability::Applicable
        );
        assert_eq!(absent.observation(), HtmlAttributeObservation::Missing);
        assert_eq!(absent.recognition(), HtmlAttributeRecognition::Missing);
        assert_eq!(absent.author(), HtmlAttributeAuthorState::Missing);
        assert_eq!(
            absent.context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
        assert_eq!(absent.raw_attribute(), None);
        assert!(absent.intrinsic().is_none());
        let attribute = dirname("");
        let empty = interpreter.dirname(&element, Some(&attribute)).unwrap();
        assert_eq!(empty.observation(), HtmlAttributeObservation::PresentEmpty);
        assert_eq!(empty.recognition(), HtmlAttributeRecognition::Empty);
        assert_eq!(
            empty.author(),
            HtmlAttributeAuthorState::Nonconforming(
                HtmlAttributeConformanceError::DirnameMustNotBeEmpty
            )
        );
        assert_eq!(empty.raw_attribute(), Some(&attribute));
        assert!(empty.intrinsic().is_none());
        assert_eq!(
            empty.context_author(),
            if element == input() {
                HtmlAttributeContextAuthorState::Unchecked
            } else {
                HtmlAttributeContextAuthorState::NotRequired
            }
        );
    }
}

#[test]
fn every_nonempty_dirname_is_exact_without_trimming_or_token_restrictions() {
    for units in [
        vec![32],
        vec![9, 10, 13, 12, 32],
        vec![0],
        vec![0xD800],
        vec![0xDC00],
        vec![0xD83D, 0xDE00],
        vec![65, 32, 0, 0xD800, 97],
    ] {
        let attribute = dirname(DocumentString::from_utf16(units.clone()));
        for element in [input(), textarea()] {
            let report = HtmlValueInterpreter::new(units.len())
                .dirname(&element, Some(&attribute))
                .unwrap();
            assert_eq!(report.raw().unwrap().code_units(), units);
            assert_eq!(report.intrinsic().unwrap().value().code_units(), units);
            assert_eq!(report.recognition(), HtmlAttributeRecognition::NonemptyText);
            assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
            assert_eq!(report.input_state(), None);
            assert_eq!(
                report.context_author(),
                if element == input() {
                    HtmlAttributeContextAuthorState::Unchecked
                } else {
                    HtmlAttributeContextAuthorState::NotRequired
                }
            );
        }
    }
}

#[test]
fn empty_placeholder_is_conforming_and_is_distinct_from_absence() {
    let interpreter = HtmlValueInterpreter::new(0);
    for element in [input(), textarea()] {
        let absent = interpreter.placeholder(&element, None).unwrap();
        assert_eq!(absent.observation(), HtmlAttributeObservation::Missing);
        assert_eq!(absent.recognition(), HtmlAttributeRecognition::Missing);
        assert_eq!(absent.author(), HtmlAttributeAuthorState::Missing);
        assert_eq!(
            absent.context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
        assert_eq!(absent.intrinsic(), None);
        let attribute = placeholder("");
        let empty = interpreter.placeholder(&element, Some(&attribute)).unwrap();
        assert_eq!(empty.observation(), HtmlAttributeObservation::PresentEmpty);
        assert_eq!(empty.recognition(), HtmlAttributeRecognition::Empty);
        assert_eq!(empty.author(), HtmlAttributeAuthorState::Conforming);
        assert_eq!(empty.raw_attribute(), Some(&attribute));
        assert_eq!(empty.intrinsic(), Some(&DocumentString::default()));
        assert_eq!(
            empty.context_author(),
            if element == input() {
                HtmlAttributeContextAuthorState::Unchecked
            } else {
                HtmlAttributeContextAuthorState::NotRequired
            }
        );
    }
}

#[test]
fn input_projection_removes_only_cr_lf_while_retaining_invalid_original_text() {
    // Independent literal strip-newlines outcomes; no whitespace normalization.
    for (original, projected, offset) in [
        ("a\nb", "ab", 1),
        ("a\rb", "ab", 1),
        ("a\r\nb", "ab", 1),
        ("\r\na\n\rb\r", "ab", 0),
        ("\n\r", "", 0),
        ("\tA\r\n B\u{c}\u{a0}", "\tA B\u{c}\u{a0}", 2),
    ] {
        let attribute = placeholder(original);
        let report = HtmlValueInterpreter::new(attribute.value().len())
            .placeholder(&input(), Some(&attribute))
            .unwrap();
        assert_eq!(report.raw_attribute(), Some(&attribute));
        assert_eq!(report.raw(), Some(&DocumentString::from(original)));
        assert_eq!(report.intrinsic(), Some(&DocumentString::from(projected)));
        assert_eq!(
            report.author(),
            HtmlAttributeAuthorState::Nonconforming(newline_error(offset))
        );
        assert_eq!(report.recognition(), HtmlAttributeRecognition::Unrecognized);
        assert_eq!(
            report.context_author(),
            HtmlAttributeContextAuthorState::Unchecked
        );
    }
}

#[test]
fn textarea_multiline_placeholder_is_conforming_and_preserves_original_break_units() {
    for original in [
        "a\nb",
        "a\rb",
        "a\r\nb",
        "\r\na\n\rb\r",
        "\n\r",
        "\t A\u{c}\u{a0}",
    ] {
        let attribute = placeholder(original);
        let report = HtmlValueInterpreter::new(attribute.value().len())
            .placeholder(&textarea(), Some(&attribute))
            .unwrap();
        assert_eq!(report.raw_attribute(), Some(&attribute));
        assert_eq!(report.intrinsic(), Some(attribute.value()));
        assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
        assert_eq!(report.recognition(), HtmlAttributeRecognition::NonemptyText);
        assert_eq!(
            report.context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
    }
}

#[test]
fn unicode_units_remain_exact_and_first_newline_offsets_use_utf16_units() {
    for units in [
        vec![0],
        vec![0xD800],
        vec![0xDC00],
        vec![0xD83D, 0xDE00],
        vec![9, 12, 0xA0, 0x2028, 0x2029],
    ] {
        let attribute = placeholder(DocumentString::from_utf16(units.clone()));
        for element in [input(), textarea()] {
            let report = HtmlValueInterpreter::new(units.len())
                .placeholder(&element, Some(&attribute))
                .unwrap();
            assert_eq!(report.raw().unwrap().code_units(), units);
            assert_eq!(report.intrinsic().unwrap().code_units(), units);
            assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
        }
    }
    let original = vec![0xD83D, 0xDE00, 0xD800, 13, 0, 10, 0xDC00];
    let attribute = placeholder(DocumentString::from_utf16(original.clone()));
    let report = HtmlValueInterpreter::new(7)
        .placeholder(&input(), Some(&attribute))
        .unwrap();
    assert_eq!(
        report.author(),
        HtmlAttributeAuthorState::Nonconforming(newline_error(3))
    );
    assert_eq!(report.raw().unwrap().code_units(), original);
    assert_eq!(
        report.intrinsic().unwrap().code_units(),
        &[0xD83D, 0xDE00, 0xD800, 0, 0xDC00]
    );
    assert_eq!(report.input_code_units(), 7);
}

#[test]
fn all_22_normative_input_states_have_independent_dirname_placeholder_presence_rules() {
    // Literal matrix from each normative bookkeeping section in HTML §4.10.5.1.
    // Text and Search share a section; both are separate source states.
    for (state, dirname_allowed, placeholder_allowed) in [
        (HtmlInputType::Hidden, true, false),
        (HtmlInputType::Text, true, true),
        (HtmlInputType::Search, true, true),
        (HtmlInputType::Telephone, true, true),
        (HtmlInputType::Url, true, true),
        (HtmlInputType::Email, true, true),
        (HtmlInputType::Password, true, true),
        (HtmlInputType::Date, false, false),
        (HtmlInputType::Month, false, false),
        (HtmlInputType::Week, false, false),
        (HtmlInputType::Time, false, false),
        (HtmlInputType::LocalDateTime, false, false),
        (HtmlInputType::Number, false, true),
        (HtmlInputType::Range, false, false),
        (HtmlInputType::Color, false, false),
        (HtmlInputType::Checkbox, false, false),
        (HtmlInputType::Radio, false, false),
        (HtmlInputType::File, false, false),
        (HtmlInputType::Submit, true, false),
        (HtmlInputType::Image, false, false),
        (HtmlInputType::Reset, false, false),
        (HtmlInputType::Button, false, false),
    ] {
        let interpreter = HtmlValueInterpreter::new(64);
        let name = dirname("Comment.Dir");
        let hint = placeholder("Enter a value");
        let name_report = interpreter
            .dirname_for_input(&input(), Some(&name), state)
            .unwrap();
        let hint_report = interpreter
            .placeholder_for_input(&input(), Some(&hint), state)
            .unwrap();
        assert_eq!(
            name_report.applicability(),
            HtmlAttributeApplicability::Applicable
        );
        assert_eq!(
            hint_report.applicability(),
            HtmlAttributeApplicability::Applicable
        );
        assert_eq!(name_report.input_state(), Some(state));
        assert_eq!(hint_report.input_state(), Some(state));
        assert_eq!(name_report.author(), HtmlAttributeAuthorState::Conforming);
        assert_eq!(hint_report.author(), HtmlAttributeAuthorState::Conforming);
        assert_eq!(name_report.intrinsic().unwrap().value(), name.value());
        assert_eq!(hint_report.intrinsic(), Some(hint.value()));
        assert_eq!(
            name_report.context_author(),
            if dirname_allowed {
                HtmlAttributeContextAuthorState::Conforming
            } else {
                forbidden()
            },
            "dirname {state:?}"
        );
        assert_eq!(
            hint_report.context_author(),
            if placeholder_allowed {
                HtmlAttributeContextAuthorState::Conforming
            } else {
                forbidden()
            },
            "placeholder {state:?}"
        );
        let absent_name = interpreter
            .dirname_for_input(&input(), None, state)
            .unwrap();
        let absent_hint = interpreter
            .placeholder_for_input(&input(), None, state)
            .unwrap();
        assert_eq!(absent_name.author(), HtmlAttributeAuthorState::Missing);
        assert_eq!(absent_hint.author(), HtmlAttributeAuthorState::Missing);
        assert_eq!(
            absent_name.context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
        assert_eq!(
            absent_hint.context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
        assert!(absent_name.intrinsic().is_none());
        assert_eq!(absent_hint.intrinsic(), None);
        assert_eq!(absent_name.input_state(), Some(state));
        assert_eq!(absent_hint.input_state(), Some(state));
    }
}

#[test]
fn lexical_failures_and_forbidden_presence_are_simultaneously_preserved() {
    let interpreter = HtmlValueInterpreter::new(16);
    let empty = dirname("");
    for (state, context) in [
        (
            HtmlInputType::Submit,
            HtmlAttributeContextAuthorState::Conforming,
        ),
        (HtmlInputType::Image, forbidden()),
    ] {
        let report = interpreter
            .dirname_for_input(&input(), Some(&empty), state)
            .unwrap();
        assert_eq!(
            report.author(),
            HtmlAttributeAuthorState::Nonconforming(
                HtmlAttributeConformanceError::DirnameMustNotBeEmpty
            )
        );
        assert_eq!(report.context_author(), context);
        assert_eq!(report.recognition(), HtmlAttributeRecognition::Empty);
        assert!(report.intrinsic().is_none());
    }
    let multiline = placeholder("a\r\nb");
    for (state, context) in [
        (
            HtmlInputType::Number,
            HtmlAttributeContextAuthorState::Conforming,
        ),
        (HtmlInputType::Hidden, forbidden()),
    ] {
        let report = interpreter
            .placeholder_for_input(&input(), Some(&multiline), state)
            .unwrap();
        assert_eq!(
            report.author(),
            HtmlAttributeAuthorState::Nonconforming(newline_error(1))
        );
        assert_eq!(report.context_author(), context);
        assert_eq!(report.raw_attribute(), Some(&multiline));
        assert_eq!(report.intrinsic(), Some(&DocumentString::from("ab")));
    }
}

#[test]
fn actual_input_type_reports_supply_states_without_implicit_state_in_content_calls() {
    let interpreter = HtmlValueInterpreter::new(64);
    let name = dirname("comment.dir");
    let hint = placeholder("comment");
    for (spelling, expected) in [
        (None, HtmlInputType::Text),
        (Some(""), HtmlInputType::Text),
        (Some("unknown"), HtmlInputType::Text),
        (Some("SuBmIt"), HtmlInputType::Submit),
    ] {
        let type_attribute =
            spelling.map(|s| Attribute::new(KnownHtmlAttribute::Type.qualified_name(), s));
        let type_report = interpreter
            .input_type(&input(), type_attribute.as_ref())
            .unwrap();
        assert_eq!(type_report.intrinsic(), Some(&expected));
        let name_report = interpreter
            .dirname_for_input(&input(), Some(&name), *type_report.intrinsic().unwrap())
            .unwrap();
        let hint_report = interpreter
            .placeholder_for_input(&input(), Some(&hint), *type_report.intrinsic().unwrap())
            .unwrap();
        assert_eq!(
            name_report.context_author(),
            HtmlAttributeContextAuthorState::Conforming
        );
        assert_eq!(
            hint_report.context_author(),
            if expected == HtmlInputType::Submit {
                forbidden()
            } else {
                HtmlAttributeContextAuthorState::Conforming
            }
        );
        assert_eq!(name_report.input_state(), Some(expected));
    }
    assert_eq!(
        interpreter
            .dirname(&input(), Some(&name))
            .unwrap()
            .input_state(),
        None
    );
    assert_eq!(
        interpreter
            .dirname(&input(), Some(&name))
            .unwrap()
            .context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
    assert_eq!(
        interpreter
            .placeholder(&input(), Some(&hint))
            .unwrap()
            .context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
}

#[test]
fn native_names_emit_exact_nonempty_text_and_reject_only_empty() {
    assert_eq!(
        HtmlDirectionSubmissionName::new(""),
        Err(HtmlAttributeConformanceError::DirnameMustNotBeEmpty)
    );
    for units in [
        vec![32],
        vec![10, 13, 9, 12],
        vec![0, 0xD800, 65, 0xD83D, 0xDE00],
    ] {
        let value =
            HtmlDirectionSubmissionName::new(DocumentString::from_utf16(units.clone())).unwrap();
        assert_eq!(value.value().code_units(), units);
        let attribute = value.into_attribute();
        assert_eq!(
            attribute.name(),
            &KnownHtmlAttribute::Dirname.qualified_name()
        );
        assert_eq!(attribute.name().namespace(), &Namespace::none());
        assert_eq!(attribute.name().prefix(), None);
        assert_eq!(attribute.value().code_units(), units);
        let report = HtmlValueInterpreter::new(units.len())
            .dirname(&textarea(), Some(&attribute))
            .unwrap();
        assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
        assert_eq!(report.intrinsic().unwrap().value().code_units(), units);
    }
}

#[test]
fn native_input_placeholders_keep_exact_text_and_reject_first_cr_or_lf() {
    for units in [
        vec![],
        vec![32, 9, 12, 0xA0],
        vec![0, 0xD800, 0xDC00, 0x2028],
    ] {
        let value = HtmlInputPlaceholder::new(DocumentString::from_utf16(units.clone())).unwrap();
        assert_eq!(value.value().code_units(), units);
        let attribute = value.into_attribute();
        assert_eq!(
            attribute.name(),
            &KnownHtmlAttribute::Placeholder.qualified_name()
        );
        assert_eq!(attribute.name().namespace(), &Namespace::none());
        assert_eq!(attribute.name().prefix(), None);
        assert_eq!(attribute.value().code_units(), units);
        let report = HtmlValueInterpreter::new(units.len())
            .placeholder_for_input(&input(), Some(&attribute), HtmlInputType::Number)
            .unwrap();
        assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
        assert_eq!(
            report.context_author(),
            HtmlAttributeContextAuthorState::Conforming
        );
        assert_eq!(report.intrinsic().unwrap().code_units(), units);
    }
    for (units, offset) in [
        (vec![10], 0),
        (vec![13], 0),
        (vec![65, 13, 10], 1),
        (vec![0xD83D, 0xDE00, 10, 13], 2),
    ] {
        assert_eq!(
            HtmlInputPlaceholder::new(DocumentString::from_utf16(units)),
            Err(newline_error(offset))
        );
    }
}

fn assert_unapplied<T>(report: &HtmlAttributeReport<T>, expected: HtmlAttributeApplicability) {
    assert_eq!(report.applicability(), expected);
    assert_eq!(report.recognition(), HtmlAttributeRecognition::Unapplied);
    assert_eq!(report.author(), HtmlAttributeAuthorState::Unapplied);
    assert_eq!(
        report.context_author(),
        HtmlAttributeContextAuthorState::Unapplied
    );
    assert!(report.intrinsic().is_none());
}

#[test]
fn wrong_expanded_roles_preserve_raw_without_text_scanning_or_projection() {
    let interpreter = HtmlValueInterpreter::new(64);
    let empty = dirname("");
    let multiline = placeholder("a\r\nb");
    for (element, application) in [
        (
            KnownHtmlElement::Div.qualified_name(),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            KnownHtmlElement::Select.qualified_name(),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            QualifiedName::new(Namespace::html(), None, "INPUT"),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            QualifiedName::new(Namespace::svg(), None, "input"),
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
        (
            QualifiedName::new(Namespace::none(), None, "textarea"),
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
    ] {
        let name_report = interpreter.dirname(&element, Some(&empty)).unwrap();
        let hint_report = interpreter.placeholder(&element, Some(&multiline)).unwrap();
        assert_unapplied(&name_report, application);
        assert_unapplied(&hint_report, application);
        assert_eq!(name_report.element(), &element);
        assert_eq!(name_report.raw_attribute(), Some(&empty));
        assert_eq!(hint_report.raw_attribute(), Some(&multiline));
    }
    for (role, attribute) in [
        ("DIRNAME", KnownHtmlAttribute::Dirname),
        ("PLACEHOLDER", KnownHtmlAttribute::Placeholder),
    ] {
        for namespace in [
            Namespace::none(),
            Namespace::html(),
            Namespace::from_uri("urn:control-text"),
        ] {
            let raw = Attribute::new(
                QualifiedName::new(namespace, Some("a".into()), role),
                "\r\n",
            );
            if attribute == KnownHtmlAttribute::Dirname {
                let report = interpreter.dirname(&input(), Some(&raw)).unwrap();
                assert_unapplied(&report, HtmlAttributeApplicability::DifferentAttribute);
                assert_eq!(report.raw_attribute(), Some(&raw));
            } else {
                let report = interpreter.placeholder(&input(), Some(&raw)).unwrap();
                assert_unapplied(&report, HtmlAttributeApplicability::DifferentAttribute);
                assert_eq!(report.raw_attribute(), Some(&raw));
            }
        }
    }
    for namespace in [Namespace::html(), Namespace::from_uri("urn:control-text")] {
        let wrong_name = Attribute::new(QualifiedName::new(namespace.clone(), None, "dirname"), "");
        let wrong_hint = Attribute::new(QualifiedName::new(namespace, None, "placeholder"), "\r\n");
        assert_unapplied(
            &interpreter.dirname(&input(), Some(&wrong_name)).unwrap(),
            HtmlAttributeApplicability::DifferentAttribute,
        );
        assert_unapplied(
            &interpreter
                .placeholder(&input(), Some(&wrong_hint))
                .unwrap(),
            HtmlAttributeApplicability::DifferentAttribute,
        );
    }
    assert_unapplied(
        &interpreter.dirname(&input(), Some(&multiline)).unwrap(),
        HtmlAttributeApplicability::DifferentAttribute,
    );
    assert_unapplied(
        &interpreter.placeholder(&input(), Some(&empty)).unwrap(),
        HtmlAttributeApplicability::DifferentAttribute,
    );
}

#[test]
fn input_only_companions_do_not_treat_a_textarea_as_supplied_input_context() {
    let interpreter = HtmlValueInterpreter::new(64);
    let name = dirname("name");
    let hint = placeholder("a\r\nb");
    for raw in [None, Some(&name)] {
        let report = interpreter
            .dirname_for_input(&textarea(), raw, HtmlInputType::Text)
            .unwrap();
        assert_unapplied(&report, HtmlAttributeApplicability::DifferentContext);
        assert_eq!(report.input_state(), Some(HtmlInputType::Text));
        assert_eq!(report.raw_attribute(), raw);
    }
    for raw in [None, Some(&hint)] {
        let report = interpreter
            .placeholder_for_input(&textarea(), raw, HtmlInputType::Text)
            .unwrap();
        assert_unapplied(&report, HtmlAttributeApplicability::DifferentContext);
        assert_eq!(report.input_state(), Some(HtmlInputType::Text));
        assert_eq!(report.raw_attribute(), raw);
    }
}

#[test]
fn prefixes_source_rows_and_context_clauses_preserve_actual_phase_evidence() {
    let element = QualifiedName::new(Namespace::html(), Some("h".into()), "input");
    let name = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("a".into()), "dirname"),
        "\tName Dir",
    );
    let hint = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("a".into()), "placeholder"),
        "a\r\nb",
    );
    let interpreter = HtmlValueInterpreter::new(64);
    let name_report = interpreter.dirname(&element, Some(&name)).unwrap();
    let hint_report = interpreter.placeholder(&element, Some(&hint)).unwrap();
    assert_eq!(name_report.element(), &element);
    assert_eq!(name_report.raw_attribute(), Some(&name));
    assert_eq!(hint_report.raw_attribute(), Some(&hint));
    assert_eq!(name_report.attribute(), KnownHtmlAttribute::Dirname);
    assert_eq!(hint_report.attribute(), KnownHtmlAttribute::Placeholder);
    assert_eq!(name_report.inventory_rows(), &[43]);
    assert_eq!(hint_report.inventory_rows(), &[117]);
    assert_eq!(
        name_report.source_clause(),
        format!("{BASE}attr-fe-dirname")
    );
    assert_eq!(
        hint_report.source_clause(),
        format!("{BASE}attr-input-placeholder")
    );
    assert_eq!(
        name_report.context_source_clause(),
        Some(format!("{BASE}the-input-element").as_str())
    );
    assert_eq!(
        hint_report.context_source_clause(),
        Some(format!("{BASE}the-input-element").as_str())
    );
    assert_eq!(name_report.vocabulary_source_clause(), None);
    assert_eq!(hint_report.vocabulary_source_clause(), None);
    for (state, anchor) in [
        (HtmlInputType::Hidden, "hidden-state-(type=hidden)"),
        (HtmlInputType::Number, "number-state-(type=number)"),
        (HtmlInputType::Submit, "submit-button-state-(type=submit)"),
    ] {
        for raw in [None, Some(&name)] {
            assert_eq!(
                interpreter
                    .dirname_for_input(&element, raw, state)
                    .unwrap()
                    .context_source_clause(),
                Some(format!("{BASE}{anchor}").as_str())
            );
        }
        assert_eq!(
            interpreter
                .placeholder_for_input(&element, Some(&hint), state)
                .unwrap()
                .context_source_clause(),
            Some(format!("{BASE}{anchor}").as_str())
        );
    }
    let multiline = interpreter.placeholder(&textarea(), Some(&hint)).unwrap();
    assert_eq!(
        multiline.source_clause(),
        format!("{BASE}attr-textarea-placeholder")
    );
    assert_eq!(
        multiline.context_source_clause(),
        Some(format!("{BASE}attr-textarea-placeholder").as_str())
    );
    assert_eq!(multiline.input_state(), None);
    assert_eq!(multiline.intrinsic(), Some(hint.value()));
    assert_eq!(
        interpreter
            .dirname(&textarea(), Some(&name))
            .unwrap()
            .context_source_clause(),
        Some(format!("{BASE}attr-fe-dirname").as_str())
    );
}

#[test]
fn raw_capacity_is_original_utf16_before_normalization_roles_or_state_companions() {
    let units = vec![0xD83D, 0xDE00, 13, 10, 0xD800];
    let name = dirname(DocumentString::from_utf16(units.clone()));
    let hint = placeholder(DocumentString::from_utf16(units.clone()));
    for element in [input(), textarea(), KnownHtmlElement::Div.qualified_name()] {
        let short = HtmlValueInterpreter::new(4);
        for error in [
            short.dirname(&element, Some(&name)).unwrap_err(),
            short.placeholder(&element, Some(&hint)).unwrap_err(),
            short
                .dirname_for_input(&element, Some(&name), HtmlInputType::Image)
                .unwrap_err(),
            short
                .placeholder_for_input(&element, Some(&hint), HtmlInputType::Hidden)
                .unwrap_err(),
        ] {
            assert_eq!(error.capacity(), 4);
            assert_eq!(error.required(), 5);
        }
        let exact = HtmlValueInterpreter::new(5);
        assert_eq!(
            exact
                .dirname(&element, Some(&name))
                .unwrap()
                .raw()
                .unwrap()
                .code_units(),
            units
        );
        assert_eq!(
            exact
                .placeholder(&element, Some(&hint))
                .unwrap()
                .raw()
                .unwrap()
                .code_units(),
            units
        );
    }
    let wrong = Attribute::new(
        KnownHtmlAttribute::Title.qualified_name(),
        DocumentString::from_utf16(units),
    );
    assert_eq!(
        HtmlValueInterpreter::new(4)
            .dirname(&input(), Some(&wrong))
            .unwrap_err()
            .required(),
        5
    );
    assert_eq!(
        HtmlValueInterpreter::new(4)
            .placeholder(&input(), Some(&wrong))
            .unwrap_err()
            .required(),
        5
    );
    let projected = HtmlValueInterpreter::new(5)
        .placeholder(&input(), Some(&hint))
        .unwrap();
    assert_eq!(
        projected.intrinsic().unwrap().code_units(),
        &[0xD83D, 0xDE00, 0xD800]
    );
    assert_eq!(projected.input_code_units(), 5);
}
