use surgeist_html::{
    Attribute, DocumentString, HtmlAttributeApplicability, HtmlAttributeAuthorState,
    HtmlAttributeConformanceError, HtmlAttributeContextAuthorState, HtmlAttributeObservation,
    HtmlAttributeRecognition, HtmlAttributeReport, HtmlSpaceToken, HtmlTokenError,
    HtmlValueInterpreter, KnownHtmlAttribute, KnownHtmlElement, Namespace, QualifiedName,
};

fn raw(value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(KnownHtmlAttribute::Id.qualified_name(), value)
}

fn read(attribute: Option<&Attribute>) -> HtmlAttributeReport<HtmlSpaceToken> {
    HtmlValueInterpreter::new(4096)
        .id_attribute(&KnownHtmlElement::Div.qualified_name(), attribute)
        .unwrap()
}

fn conforming(report: &HtmlAttributeReport<HtmlSpaceToken>, value: &DocumentString) {
    assert_eq!(
        report.applicability(),
        HtmlAttributeApplicability::Applicable
    );
    assert_eq!(
        report.observation(),
        HtmlAttributeObservation::PresentNonempty
    );
    assert_eq!(report.recognition(), HtmlAttributeRecognition::NonemptyText);
    assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
    assert_eq!(
        report.context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
    assert_eq!(report.raw(), Some(value));
    assert_eq!(report.intrinsic().unwrap().value(), value);
}

fn nonconforming(attribute: &Attribute, error: HtmlTokenError) {
    let report = read(Some(attribute));
    assert_eq!(
        report.applicability(),
        HtmlAttributeApplicability::Applicable
    );
    assert_eq!(report.raw_attribute(), Some(attribute));
    assert_eq!(report.raw(), Some(attribute.value()));
    assert_eq!(report.intrinsic(), None);
    assert_eq!(
        report.context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
    assert_eq!(
        report.author(),
        HtmlAttributeAuthorState::Nonconforming(HtmlAttributeConformanceError::InvalidIdToken(
            error
        ))
    );
}

#[test]
fn optional_absence_differs_from_specified_empty_local_author_failure() {
    // #the-id-attribute requires at least one character only when specified.
    let missing = read(None);
    assert_eq!(
        missing.applicability(),
        HtmlAttributeApplicability::Applicable
    );
    assert_eq!(missing.observation(), HtmlAttributeObservation::Missing);
    assert_eq!(missing.recognition(), HtmlAttributeRecognition::Missing);
    assert_eq!(missing.author(), HtmlAttributeAuthorState::Missing);
    assert_eq!(
        missing.context_author(),
        HtmlAttributeContextAuthorState::NotRequired
    );
    assert_eq!(missing.raw_attribute(), None);
    assert_eq!(missing.intrinsic(), None);

    let empty = raw("");
    let report = read(Some(&empty));
    assert_eq!(report.observation(), HtmlAttributeObservation::PresentEmpty);
    assert_eq!(report.recognition(), HtmlAttributeRecognition::Empty);
    nonconforming(&empty, HtmlTokenError::EmptySpaceToken);
    assert_eq!(
        HtmlSpaceToken::new(""),
        Err(HtmlTokenError::EmptySpaceToken)
    );
}

#[test]
fn digits_punctuation_and_other_opaque_forms_are_exact_local_author_values() {
    // The source expressly permits digits, leading digits, underscores and punctuation.
    for text in [
        "123", "1x", "_", "!", "A", "a", "id", "ID", ",", "#", "a:b", "é", "e\u{301}",
    ] {
        let attribute = raw(text);
        conforming(&read(Some(&attribute)), &DocumentString::from(text));
        let emitted = HtmlSpaceToken::new(text).unwrap().into_id_attribute();
        assert_eq!(emitted.name(), &KnownHtmlAttribute::Id.qualified_name());
        assert_eq!(emitted.name().namespace(), &Namespace::none());
        assert_eq!(emitted.name().prefix(), None);
        assert_eq!(emitted.name().local_name(), &DocumentString::from("id"));
        assert_eq!(emitted.value(), &DocumentString::from(text));
        conforming(&read(Some(&emitted)), emitted.value());
    }
    let uppercase = raw("A");
    let lowercase = raw("a");
    assert_ne!(
        read(Some(&uppercase)).intrinsic(),
        read(Some(&lowercase)).intrinsic()
    );
    let composed = raw("é");
    let decomposed = raw("e\u{301}");
    assert_ne!(
        read(Some(&composed)).intrinsic(),
        read(Some(&decomposed)).intrinsic()
    );
}

#[test]
fn all_ascii_whitespace_is_rejected_without_trimming_or_token_splitting() {
    for whitespace in [0x09, 0x0A, 0x0C, 0x0D, 0x20] {
        for (units, offset) in [
            (vec![whitespace], 0),
            (vec![whitespace, 0x78], 0),
            (vec![0x78, whitespace], 1),
            (vec![0x78, whitespace, 0x79], 1),
            (vec![0x78, 0x79, whitespace, 0x20], 2),
        ] {
            let value = DocumentString::from_utf16(units.clone());
            let attribute = raw(value.clone());
            nonconforming(
                &attribute,
                HtmlTokenError::SpaceTokenWhitespace {
                    code_unit_offset: offset,
                },
            );
            let report = read(Some(&attribute));
            assert_eq!(
                report.observation(),
                HtmlAttributeObservation::PresentNonempty
            );
            assert_eq!(report.recognition(), HtmlAttributeRecognition::Unrecognized);
            assert_eq!(report.raw().unwrap().code_units(), units);
            assert_eq!(
                HtmlSpaceToken::new(value),
                Err(HtmlTokenError::SpaceTokenWhitespace {
                    code_unit_offset: offset
                })
            );
        }
    }
    let multiple = raw("x\t y\n");
    nonconforming(
        &multiple,
        HtmlTokenError::SpaceTokenWhitespace {
            code_unit_offset: 1,
        },
    );
}

#[test]
fn non_ascii_whitespace_null_and_surrogates_remain_exact_native_and_raw_values() {
    for units in [
        vec![0xA0],
        vec![0x2003],
        vec![0],
        vec![0xD800],
        vec![0xDC00],
        vec![0xD83D, 0xDE00],
        vec![0x78, 0xA0, 0, 0xD800, 0xDC00, 0x79],
    ] {
        let value = DocumentString::from_utf16(units.clone());
        let attribute = raw(value.clone());
        conforming(&read(Some(&attribute)), &value);
        assert_eq!(read(Some(&attribute)).raw().unwrap().code_units(), units);
        let emitted = HtmlSpaceToken::new(value).unwrap().into_id_attribute();
        assert_eq!(emitted.value().code_units(), units);
        conforming(&read(Some(&emitted)), emitted.value());
    }
}

#[test]
fn first_whitespace_offset_counts_utf16_units_including_isolated_surrogates() {
    for (units, offset) in [
        (vec![0xD800, 0x20], 1),
        (vec![0xDC00, 0x09, 0x20], 1),
        (vec![0xD83D, 0xDE00, 0x20], 2),
        (vec![0, 0xA0, 0xD800, 0x0D], 3),
    ] {
        let attribute = raw(DocumentString::from_utf16(units));
        nonconforming(
            &attribute,
            HtmlTokenError::SpaceTokenWhitespace {
                code_unit_offset: offset,
            },
        );
    }
}

#[test]
fn source_metadata_and_exact_prefix_records_follow_expanded_names() {
    let element = QualifiedName::new(Namespace::html(), Some("h".into()), "div");
    let attribute = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("p".into()), "id"),
        "123",
    );
    let report = HtmlValueInterpreter::new(64)
        .id_attribute(&element, Some(&attribute))
        .unwrap();
    conforming(&report, &DocumentString::from("123"));
    assert_eq!(report.element(), &element);
    assert_eq!(report.raw_attribute(), Some(&attribute));
    assert_eq!(report.attribute(), KnownHtmlAttribute::Id);
    assert_eq!(report.inventory_rows(), &[72]);
    assert_eq!(
        report.source_clause(),
        "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-id-attribute"
    );
    assert_eq!(report.vocabulary_source_clause(), None);
    assert_eq!(report.script_context(), None);
    assert_eq!(report.hyperlink_context(), None);
    let emitted = report.intrinsic().unwrap().clone().into_id_attribute();
    assert_eq!(emitted.name().prefix(), None);
    assert_eq!(report.raw_attribute(), Some(&attribute));
}

#[test]
fn local_global_rule_applies_to_known_and_open_html_element_names() {
    for element in [
        KnownHtmlElement::Div.qualified_name(),
        KnownHtmlElement::Details.qualified_name(),
        KnownHtmlElement::Input.qualified_name(),
        QualifiedName::new(Namespace::html(), None, "x-widget"),
        QualifiedName::new(Namespace::html(), Some("h".into()), "unknown"),
    ] {
        let attribute = raw("opaque");
        let report = HtmlValueInterpreter::new(64)
            .id_attribute(&element, Some(&attribute))
            .unwrap();
        conforming(&report, &DocumentString::from("opaque"));
        assert_eq!(report.element(), &element);
    }
}

#[test]
fn wrong_role_or_namespace_preserves_unapplied_raw_without_local_validation() {
    let id = raw("");
    let div = KnownHtmlElement::Div.qualified_name();
    let wrong = Attribute::new(KnownHtmlAttribute::Name.qualified_name(), " x ");
    let uppercase = Attribute::new(QualifiedName::new(Namespace::none(), None, "ID"), "");
    let namespaced = Attribute::new(
        QualifiedName::new(Namespace::svg(), Some("s".into()), "id"),
        " x ",
    );
    for (element, attribute, applicability) in [
        (
            QualifiedName::new(Namespace::svg(), None, "div"),
            &id,
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
        (
            QualifiedName::new(Namespace::mathml(), None, "div"),
            &id,
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
        (
            QualifiedName::new(Namespace::none(), None, "div"),
            &id,
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
        (
            div.clone(),
            &wrong,
            HtmlAttributeApplicability::DifferentAttribute,
        ),
        (
            div.clone(),
            &uppercase,
            HtmlAttributeApplicability::DifferentAttribute,
        ),
        (
            div,
            &namespaced,
            HtmlAttributeApplicability::DifferentAttribute,
        ),
    ] {
        let report = HtmlValueInterpreter::new(64)
            .id_attribute(&element, Some(attribute))
            .unwrap();
        assert_eq!(report.applicability(), applicability);
        assert_eq!(report.element(), &element);
        assert_eq!(report.raw_attribute(), Some(attribute));
        assert_eq!(report.raw(), Some(attribute.value()));
        assert_eq!(report.author(), HtmlAttributeAuthorState::Unapplied);
        assert_eq!(report.recognition(), HtmlAttributeRecognition::Unapplied);
        assert_eq!(
            report.context_author(),
            HtmlAttributeContextAuthorState::Unapplied
        );
        assert_eq!(report.intrinsic(), None);
        assert_eq!(
            report.observation(),
            if attribute.value().is_empty() {
                HtmlAttributeObservation::PresentEmpty
            } else {
                HtmlAttributeObservation::PresentNonempty
            }
        );
    }
    let absent = HtmlValueInterpreter::new(0)
        .id_attribute(&QualifiedName::new(Namespace::svg(), None, "div"), None)
        .unwrap();
    assert_eq!(absent.observation(), HtmlAttributeObservation::Missing);
    assert_eq!(absent.recognition(), HtmlAttributeRecognition::Unapplied);
    assert_eq!(absent.author(), HtmlAttributeAuthorState::Unapplied);
    assert_eq!(
        absent.context_author(),
        HtmlAttributeContextAuthorState::Unapplied
    );
    assert_eq!(absent.intrinsic(), None);
}

#[test]
fn raw_utf16_admission_precedes_wrong_role_and_invalid_value_checks() {
    let div = KnownHtmlElement::Div.qualified_name();
    for units in [vec![0xD800], vec![0xD83D, 0xDE00], vec![0xD800, 0x20, 0]] {
        let value = DocumentString::from_utf16(units.clone());
        for (element, name) in [
            (div.clone(), KnownHtmlAttribute::Id.qualified_name()),
            (div.clone(), KnownHtmlAttribute::Name.qualified_name()),
            (
                QualifiedName::new(Namespace::svg(), None, "div"),
                KnownHtmlAttribute::Id.qualified_name(),
            ),
        ] {
            let attribute = Attribute::new(name, value.clone());
            let error = HtmlValueInterpreter::new(units.len() - 1)
                .id_attribute(&element, Some(&attribute))
                .unwrap_err();
            assert_eq!(error.capacity(), units.len() - 1);
            assert_eq!(error.required(), units.len());
            let admitted = HtmlValueInterpreter::new(units.len())
                .id_attribute(&element, Some(&attribute))
                .unwrap();
            assert_eq!(admitted.raw_attribute(), Some(&attribute));
            assert_eq!(admitted.raw().unwrap().code_units(), units);
        }
    }
    let interpreter = HtmlValueInterpreter::new(0);
    assert_eq!(
        interpreter.id_attribute(&div, None).unwrap().author(),
        HtmlAttributeAuthorState::Missing
    );
    let empty = raw("");
    assert_eq!(
        interpreter
            .id_attribute(&div, Some(&empty))
            .unwrap()
            .author(),
        HtmlAttributeAuthorState::Nonconforming(HtmlAttributeConformanceError::InvalidIdToken(
            HtmlTokenError::EmptySpaceToken
        ))
    );
    let whitespace = raw(" ");
    let error = interpreter
        .id_attribute(&div, Some(&whitespace))
        .unwrap_err();
    assert_eq!(error.capacity(), 0);
    assert_eq!(error.required(), 1);
}

#[test]
fn long_admitted_id_is_not_truncated_and_reports_survive_later_operations() {
    let value = DocumentString::from("opaque1!".repeat(512));
    let attribute = raw(value.clone());
    let report = read(Some(&attribute));
    conforming(&report, &value);
    let error = HtmlValueInterpreter::new(4095)
        .id_attribute(&KnownHtmlElement::Div.qualified_name(), Some(&attribute))
        .unwrap_err();
    assert_eq!(error.required(), 4096);
    assert_eq!(error.capacity(), 4095);
    let retained = report.clone();
    drop(attribute);
    nonconforming(
        &raw("later id"),
        HtmlTokenError::SpaceTokenWhitespace {
            code_unit_offset: 5,
        },
    );
    assert_eq!(report, retained);
    let emitted = report.intrinsic().unwrap().clone().into_id_attribute();
    assert_eq!(emitted.value(), &value);
    conforming(&read(Some(&emitted)), &value);
}

#[test]
fn equal_local_id_values_do_not_establish_tree_uniqueness() {
    let attribute = raw("duplicate");
    let interpreter = HtmlValueInterpreter::new(64);
    let first = interpreter
        .id_attribute(&KnownHtmlElement::Div.qualified_name(), Some(&attribute))
        .unwrap();
    let second = interpreter
        .id_attribute(&KnownHtmlElement::Span.qualified_name(), Some(&attribute))
        .unwrap();
    conforming(&first, &DocumentString::from("duplicate"));
    conforming(&second, &DocumentString::from("duplicate"));
    assert_eq!(first.intrinsic(), second.intrinsic());
    assert_ne!(first.element(), second.element());
    assert_eq!(
        first.context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
    assert_eq!(
        second.context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
}
