use surgeist_html::{
    Attribute, DocumentString, HtmlAcceptAuthorState, HtmlAcceptMimeConformance,
    HtmlAcceptProcessingState, HtmlAcceptReport, HtmlAcceptSpecifier, HtmlAcceptSpecifierError,
    HtmlAcceptSpecifierKind, HtmlAcceptValue, HtmlAttributeApplicability,
    HtmlAttributeContextAuthorState, HtmlAttributeContextConformanceError,
    HtmlAttributeObservation, HtmlCommaToken, HtmlInputType, HtmlTokenError, HtmlValueInterpreter,
    KnownHtmlAttribute, KnownHtmlElement, Namespace, QualifiedName,
};

fn raw(value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(KnownHtmlAttribute::Accept.qualified_name(), value)
}
fn file(value: impl Into<DocumentString>) -> HtmlAcceptReport {
    HtmlValueInterpreter::new(1000)
        .input_accept_for_input(
            &KnownHtmlElement::Input.qualified_name(),
            Some(&raw(value)),
            HtmlInputType::File,
        )
        .unwrap()
}
fn strings(values: Option<&[HtmlAcceptSpecifier]>) -> Vec<String> {
    values
        .unwrap()
        .iter()
        .map(|value| value.value().to_utf8().unwrap())
        .collect()
}
fn specifier(value: &str) -> HtmlAcceptSpecifier {
    HtmlAcceptSpecifier::new(HtmlCommaToken::new(value).unwrap())
}

#[test]
fn native_wildcards_and_any_leading_dot_preserve_exact_tokens() {
    let value = HtmlAcceptValue::new([
        HtmlAcceptSpecifier::audio(),
        HtmlAcceptSpecifier::video(),
        HtmlAcceptSpecifier::image(),
        HtmlAcceptSpecifier::extension(".").unwrap(),
        HtmlAcceptSpecifier::extension(".tar.gz").unwrap(),
        HtmlAcceptSpecifier::extension(".x y").unwrap(),
        HtmlAcceptSpecifier::extension(".a/b;*").unwrap(),
    ])
    .unwrap();
    assert_eq!(
        value.as_document_string().to_utf8().unwrap(),
        "audio/*,video/*,image/*,.,.tar.gz,.x y,.a/b;*"
    );
    assert_eq!(
        value.mime_conformance(),
        HtmlAcceptMimeConformance::NotRequired
    );
    let exact = DocumentString::from_utf16(vec![0x2e, 0xd800, 0, 0xdc00, 0xd83d, 0xde00]);
    let extension = HtmlAcceptSpecifier::extension(exact.clone()).unwrap();
    assert_eq!(extension.kind(), HtmlAcceptSpecifierKind::Extension);
    assert_eq!(extension.value(), &exact);
    assert_eq!(
        HtmlAcceptValue::new([extension])
            .unwrap()
            .as_document_string(),
        exact
    );
}

#[test]
fn native_extension_checks_only_comma_token_grammar_and_leading_dot() {
    assert_eq!(
        HtmlAcceptSpecifier::extension("txt"),
        Err(HtmlAcceptSpecifierError::MissingLeadingDot)
    );
    assert_eq!(
        HtmlAcceptSpecifier::extension(" .txt"),
        Err(HtmlAcceptSpecifierError::Token(
            HtmlTokenError::CommaTokenBoundaryWhitespace {
                code_unit_offset: 0
            }
        ))
    );
    assert_eq!(
        HtmlAcceptSpecifier::extension(".txt "),
        Err(HtmlAcceptSpecifierError::Token(
            HtmlTokenError::CommaTokenBoundaryWhitespace {
                code_unit_offset: 4
            }
        ))
    );
    assert_eq!(
        HtmlAcceptSpecifier::extension(".x,.y"),
        Err(HtmlAcceptSpecifierError::Token(
            HtmlTokenError::CommaTokenComma {
                code_unit_offset: 2
            }
        ))
    );
}

#[test]
fn native_uniqueness_folds_only_ascii_and_reports_exact_duplicate() {
    for (first, second) in [
        (".TXT", ".txt"),
        ("IMAGE/*", "image/*"),
        ("image/PNG", "IMAGE/png"),
    ] {
        let duplicate =
            HtmlAcceptValue::new([specifier(first), specifier(".other"), specifier(second)])
                .unwrap_err();
        assert_eq!(duplicate.first_index(), 0);
        assert_eq!(duplicate.duplicate_index(), 2);
        assert_eq!(duplicate.token(), &DocumentString::from(second));
    }
    let value = HtmlAcceptValue::new([specifier(".Ä"), specifier(".ä")]).unwrap();
    assert_eq!(value.as_document_string().to_utf8().unwrap(), ".Ä,.ä");
}

#[test]
fn symbolic_mime_construction_never_certifies_external_conformance() {
    for text in [
        "image/png",
        "image/png; charset=utf-8",
        "text/*",
        "*/*",
        "not a MIME",
        "",
        "x\0y",
    ] {
        let value = HtmlAcceptValue::new([specifier(text)]).unwrap();
        assert_eq!(
            value.specifiers()[0].kind(),
            HtmlAcceptSpecifierKind::SymbolicMime
        );
        assert_eq!(
            value.mime_conformance(),
            HtmlAcceptMimeConformance::Unavailable
        );
        assert_eq!(value.as_document_string(), DocumentString::from(text));
    }
    let exact = DocumentString::from_utf16(vec![0xd800, 0x2f, 0xdc00]);
    let token = HtmlAcceptSpecifier::new(HtmlCommaToken::new(exact.clone()).unwrap());
    let value = HtmlAcceptValue::new([token]).unwrap();
    assert_eq!(value.as_document_string(), exact);
    assert_eq!(
        value.mime_conformance(),
        HtmlAcceptMimeConformance::Unavailable
    );
}

#[test]
fn missing_empty_and_whitespace_have_independent_presence_and_fields() {
    let missing = HtmlValueInterpreter::new(100)
        .input_accept_for_input(
            &KnownHtmlElement::Input.qualified_name(),
            None::<&surgeist_html::Attribute>,
            HtmlInputType::File,
        )
        .unwrap();
    assert_eq!(missing.observation(), HtmlAttributeObservation::Missing);
    assert_eq!(missing.author(), &HtmlAcceptAuthorState::Missing);
    assert_eq!(missing.author_fields(), None);
    assert_eq!(missing.processing_tokens(), None);
    assert_eq!(
        missing.processing(),
        HtmlAcceptProcessingState::MissingAttribute
    );
    assert_eq!(
        missing.context_author(),
        HtmlAttributeContextAuthorState::NotRequired
    );
    let empty = file("");
    assert_eq!(empty.observation(), HtmlAttributeObservation::PresentEmpty);
    assert_eq!(empty.author(), &HtmlAcceptAuthorState::Conforming);
    assert!(empty.author_fields().unwrap().is_empty());
    assert!(empty.processing_tokens().unwrap().is_empty());
    assert_eq!(
        empty.processing(),
        HtmlAcceptProcessingState::FileHintTokens
    );
    let white = file(" \t\n\r\u{c}");
    assert_eq!(
        white.observation(),
        HtmlAttributeObservation::PresentNonempty
    );
    assert!(white.author_fields().unwrap().is_empty());
    assert_eq!(strings(white.processing_tokens()), [""]);
    assert_eq!(white.author(), &HtmlAcceptAuthorState::Conforming);
    assert_eq!(
        white.mime_conformance(),
        HtmlAcceptMimeConformance::NotRequired
    );
    assert_eq!(
        white.processing_mime_conformance(),
        HtmlAcceptMimeConformance::Unavailable
    );
}

#[test]
fn authored_terminal_empty_is_retained_without_changing_infra_split() {
    for (text, authored, processed) in [
        ("audio/*,", vec!["audio/*", ""], vec!["audio/*"]),
        ("audio/*, ", vec!["audio/*", ""], vec!["audio/*", ""]),
        (",", vec!["", ""], vec![""]),
        (",,", vec!["", "", ""], vec!["", ""]),
        (
            " .a ,image/*,,.b ",
            vec![".a", "image/*", "", ".b"],
            vec![".a", "image/*", "", ".b"],
        ),
    ] {
        let report = file(text);
        assert_eq!(strings(report.author_fields()), authored);
        assert_eq!(strings(report.processing_tokens()), processed);
        assert_eq!(report.raw(), Some(&DocumentString::from(text)));
        assert_eq!(
            report.mime_conformance(),
            HtmlAcceptMimeConformance::Unavailable
        );
    }
    assert_eq!(
        file("audio/*,").author(),
        &HtmlAcceptAuthorState::RequiresMimeConformance
    );
    let comma = file(",");
    let HtmlAcceptAuthorState::Nonconforming(duplicate) = comma.author() else {
        panic!("duplicate authored empties");
    };
    assert_eq!(
        (duplicate.first_index(), duplicate.duplicate_index()),
        (0, 1)
    );
    assert_eq!(comma.processing_duplicate(), None);
}

#[test]
fn raw_roles_trim_only_ascii_boundaries_and_preserve_exact_source() {
    let text = " \tAUDIO/* ,Video/*, IMAGE/*, .x y ,\u{a0}.z,.é ";
    let report = file(text);
    let roles: Vec<_> = report
        .processing_tokens()
        .unwrap()
        .iter()
        .map(HtmlAcceptSpecifier::kind)
        .collect();
    assert_eq!(
        roles,
        [
            HtmlAcceptSpecifierKind::Audio,
            HtmlAcceptSpecifierKind::Video,
            HtmlAcceptSpecifierKind::Image,
            HtmlAcceptSpecifierKind::Extension,
            HtmlAcceptSpecifierKind::SymbolicMime,
            HtmlAcceptSpecifierKind::Extension
        ]
    );
    assert_eq!(
        report.author(),
        &HtmlAcceptAuthorState::RequiresMimeConformance
    );
    assert_eq!(report.raw(), Some(&DocumentString::from(text)));
    let raw = DocumentString::from_utf16(vec![0x20, 0x2e, 0xd800, 0, 0xdc00, 0x2c, 0xd83d, 0xde00]);
    let report = file(raw.clone());
    assert_eq!(
        report.processing_tokens().unwrap()[0].value().code_units(),
        &[0x2e, 0xd800, 0, 0xdc00]
    );
    assert_eq!(
        report.processing_tokens().unwrap()[1].value().code_units(),
        &[0xd83d, 0xde00]
    );
    assert_eq!(report.raw(), Some(&raw));
}

#[test]
fn duplicate_failure_does_not_hide_unavailable_mime_or_repair_hint_tokens() {
    let report = file(".TXT,image/png,.txt");
    let HtmlAcceptAuthorState::Nonconforming(duplicate) = report.author() else {
        panic!("known duplicate");
    };
    assert_eq!(
        (duplicate.first_index(), duplicate.duplicate_index()),
        (0, 2)
    );
    assert_eq!(duplicate.token(), &DocumentString::from(".txt"));
    assert_eq!(
        report.mime_conformance(),
        HtmlAcceptMimeConformance::Unavailable
    );
    assert_eq!(
        strings(report.processing_tokens()),
        [".TXT", "image/png", ".txt"]
    );
    assert_eq!(report.processing_duplicate(), Some(duplicate));
    assert_eq!(
        report.processing(),
        HtmlAcceptProcessingState::FileHintTokens
    );
}

#[test]
fn actual_nonfile_states_ignore_effect_but_keep_intrinsic_and_presence_constraints() {
    use HtmlInputType::*;
    for state in [
        Hidden,
        Text,
        Search,
        Telephone,
        Url,
        Email,
        Password,
        Date,
        Month,
        Week,
        Time,
        LocalDateTime,
        Number,
        Range,
        Color,
        Checkbox,
        Radio,
        Submit,
        Image,
        Reset,
        Button,
    ] {
        let report = HtmlValueInterpreter::new(100)
            .input_accept_for_input(
                &KnownHtmlElement::Input.qualified_name(),
                Some(&raw(".txt")),
                state,
            )
            .unwrap();
        assert_eq!(
            report.applicability(),
            HtmlAttributeApplicability::Applicable
        );
        assert_eq!(report.author(), &HtmlAcceptAuthorState::Conforming);
        assert_eq!(strings(report.processing_tokens()), [".txt"]);
        assert_eq!(
            report.context_author(),
            HtmlAttributeContextAuthorState::Nonconforming(
                HtmlAttributeContextConformanceError::InputStateForbidsAttribute
            )
        );
        assert_eq!(
            report.processing(),
            HtmlAcceptProcessingState::IgnoredForInputState(state)
        );
        assert_eq!(report.input_state(), Some(state));
    }
}

#[test]
fn unsupplied_state_is_required_and_type_defaults_do_not_guess_file() {
    let interpreter = HtmlValueInterpreter::new(100);
    let element = KnownHtmlElement::Input.qualified_name();
    let report = interpreter
        .input_accept(&element, Some(&raw(".txt")))
        .unwrap();
    assert_eq!(
        report.processing(),
        HtmlAcceptProcessingState::InputStateRequired
    );
    assert_eq!(
        report.context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
    assert_eq!(report.author(), &HtmlAcceptAuthorState::Conforming);
    let state = *interpreter
        .input_type(&element, None::<&surgeist_html::Attribute>)
        .unwrap()
        .intrinsic()
        .unwrap();
    assert_eq!(state, HtmlInputType::Text);
    assert_eq!(
        interpreter
            .input_accept_for_input(&element, Some(&raw(".txt")), state)
            .unwrap()
            .processing(),
        HtmlAcceptProcessingState::IgnoredForInputState(HtmlInputType::Text)
    );
}

#[test]
fn expanded_names_control_applicability_and_retain_raw_when_unapplied() {
    let interpreter = HtmlValueInterpreter::new(100);
    let attribute = raw(".txt");
    let prefixed = QualifiedName::new(Namespace::html(), Some("h".into()), "input");
    assert_eq!(
        interpreter
            .input_accept_for_input(&prefixed, Some(&attribute), HtmlInputType::File)
            .unwrap()
            .processing(),
        HtmlAcceptProcessingState::FileHintTokens
    );
    for (element, attribute, expected) in [
        (
            KnownHtmlElement::Div.qualified_name(),
            attribute.clone(),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            QualifiedName::new(Namespace::svg(), None, "input"),
            attribute.clone(),
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
        (
            KnownHtmlElement::Input.qualified_name(),
            Attribute::new(QualifiedName::new(Namespace::svg(), None, "accept"), ".txt"),
            HtmlAttributeApplicability::DifferentAttribute,
        ),
    ] {
        let report = interpreter
            .input_accept_for_input(&element, Some(&attribute), HtmlInputType::File)
            .unwrap();
        assert_eq!(report.applicability(), expected);
        assert_eq!(report.author(), &HtmlAcceptAuthorState::Unapplied);
        assert_eq!(report.processing(), HtmlAcceptProcessingState::Unapplied);
        assert_eq!(report.author_fields(), None);
        assert_eq!(report.raw_attribute(), Some(&attribute));
    }
}

#[test]
fn raw_admission_counts_untrimmed_utf16_before_name_or_state_processing() {
    let raw_value = DocumentString::from_utf16(vec![0x20, 0xd83d, 0xde00, 0xd800, 0x20]);
    let attribute = raw(raw_value.clone());
    let error = HtmlValueInterpreter::new(4)
        .input_accept_for_input(
            &KnownHtmlElement::Div.qualified_name(),
            Some(&attribute),
            HtmlInputType::File,
        )
        .unwrap_err();
    assert_eq!(error.capacity(), 4);
    assert_eq!(error.required(), 5);
    let report = HtmlValueInterpreter::new(5)
        .input_accept_for_input(
            &KnownHtmlElement::Input.qualified_name(),
            Some(&attribute),
            HtmlInputType::File,
        )
        .unwrap();
    assert_eq!(report.raw(), Some(&raw_value));
    assert_eq!(report.input_code_units(), 5);
    assert_eq!(report.inventory_rows(), &[2]);
    assert_eq!(report.attribute(), KnownHtmlAttribute::Accept);
    assert!(report.source_clause().ends_with("#attr-input-accept"));
    assert!(
        report
            .mime_source_clause()
            .ends_with("#valid-mime-type-with-no-parameters")
    );
    assert!(
        report
            .context_source_clause()
            .unwrap()
            .ends_with("#file-upload-state-(type=file)")
    );
}

#[test]
fn native_zero_set_and_empty_symbolic_token_retain_distinct_conformance() {
    let zero = HtmlAcceptValue::new([]).unwrap();
    let one = HtmlAcceptValue::new([specifier("")]).unwrap();
    assert!(zero.specifiers().is_empty());
    assert_eq!(one.specifiers().len(), 1);
    assert_ne!(zero, one);
    assert_eq!(zero.as_document_string(), DocumentString::from(""));
    assert_eq!(one.as_document_string(), zero.as_document_string());
    assert_eq!(
        zero.mime_conformance(),
        HtmlAcceptMimeConformance::NotRequired
    );
    assert_eq!(
        one.mime_conformance(),
        HtmlAcceptMimeConformance::Unavailable
    );
}

#[test]
fn missing_accept_in_nonfile_state_has_no_authored_presence_violation() {
    let report = HtmlValueInterpreter::new(0)
        .input_accept_for_input(
            &KnownHtmlElement::Input.qualified_name(),
            None::<&surgeist_html::Attribute>,
            HtmlInputType::Text,
        )
        .unwrap();
    assert_eq!(
        report.context_author(),
        HtmlAttributeContextAuthorState::NotRequired
    );
    assert_eq!(report.author(), &HtmlAcceptAuthorState::Missing);
    assert_eq!(
        report.processing(),
        HtmlAcceptProcessingState::IgnoredForInputState(HtmlInputType::Text)
    );
    assert_eq!(
        report.mime_conformance(),
        HtmlAcceptMimeConformance::NotRequired
    );
    assert_eq!(
        report.processing_mime_conformance(),
        HtmlAcceptMimeConformance::NotRequired
    );
}

#[test]
fn raw_symbolic_mime_is_not_rejected_or_defaulted_without_its_external_grammar() {
    for value in [
        "image/png; charset=utf-8",
        "text/*",
        "*/*",
        "not a MIME",
        "x\0y",
    ] {
        let report = file(value);
        assert_eq!(
            report.author(),
            &HtmlAcceptAuthorState::RequiresMimeConformance
        );
        assert_eq!(
            report.mime_conformance(),
            HtmlAcceptMimeConformance::Unavailable
        );
        assert_eq!(
            report.processing_mime_conformance(),
            HtmlAcceptMimeConformance::Unavailable
        );
        assert_eq!(
            report.processing_tokens().unwrap()[0].kind(),
            HtmlAcceptSpecifierKind::SymbolicMime
        );
        assert_eq!(
            report.processing(),
            HtmlAcceptProcessingState::FileHintTokens
        );
        assert_eq!(report.raw(), Some(&DocumentString::from(value)));
    }
    let attribute = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("x".into()), "accept"),
        "image/*",
    );
    let report = HtmlValueInterpreter::new(100)
        .input_accept_for_input(
            &KnownHtmlElement::Input.qualified_name(),
            Some(&attribute),
            HtmlInputType::File,
        )
        .unwrap();
    assert_eq!(report.author(), &HtmlAcceptAuthorState::Conforming);
    assert_eq!(report.raw_attribute(), Some(&attribute));
    assert_eq!(
        report.processing_tokens().unwrap()[0].kind(),
        HtmlAcceptSpecifierKind::Image
    );
}
