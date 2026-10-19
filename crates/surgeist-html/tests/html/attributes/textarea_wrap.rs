use crate::html::support::known_attribute as raw;
use surgeist_html::{
    Attribute, DocumentString, HtmlAttributeApplicability, HtmlAttributeAuthorState,
    HtmlAttributeConformanceError, HtmlAttributeContextAuthorState,
    HtmlAttributeContextConformanceError, HtmlAttributeObservation, HtmlAttributeRecognition,
    HtmlAttributeReport, HtmlNumericAttributeAuthorState, HtmlNumericAttributeConformanceError,
    HtmlNumericAttributeValueOrigin, HtmlTextareaWrap as Wrap, HtmlValueAuthorState,
    HtmlValueInterpreter, KnownHtmlAttribute as Attr, KnownHtmlElement as Element, Namespace,
    OrderedAttributes, QualifiedName,
};

const SOURCE: &str = "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-textarea-wrap";

fn attributes(values: Vec<Attribute>) -> OrderedAttributes {
    OrderedAttributes::new(values).unwrap()
}
fn content(value: Option<&Attribute>) -> HtmlAttributeReport<Wrap> {
    HtmlValueInterpreter::new(8192)
        .textarea_wrap(&Element::Textarea.qualified_name(), value)
        .unwrap()
}
fn companion(attrs: &OrderedAttributes) -> HtmlAttributeReport<Wrap> {
    HtmlValueInterpreter::new(8192)
        .textarea_wrap_with_cols(&Element::Textarea.qualified_name(), attrs)
        .unwrap()
}
fn assert_hard(report: &HtmlAttributeReport<Wrap>) {
    assert_eq!(
        report.applicability(),
        HtmlAttributeApplicability::Applicable
    );
    assert_eq!(report.intrinsic(), Some(&Wrap::Hard));
    assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
    assert_eq!(
        report.recognition(),
        HtmlAttributeRecognition::Keyword {
            matched: "hard",
            canonical: "hard"
        }
    );
}

#[test]
fn canonical_native_keywords_emit_real_attributes_without_certifying_cols_context() {
    // #attr-textarea-wrap defines two conforming keywords; neither constructs a submission value.
    for (native, canonical, context) in [
        (
            Wrap::Soft,
            "soft",
            HtmlAttributeContextAuthorState::NotRequired,
        ),
        (
            Wrap::Hard,
            "hard",
            HtmlAttributeContextAuthorState::Unchecked,
        ),
    ] {
        assert_eq!(native.canonical_keyword(), canonical);
        let attribute = native.into_attribute();
        assert_eq!(attribute.name(), &Attr::Wrap.qualified_name());
        assert_eq!(attribute.name().namespace(), &Namespace::none());
        assert_eq!(attribute.name().prefix(), None);
        assert_eq!(attribute.value(), &DocumentString::from(canonical));
        let report = content(Some(&attribute));
        assert_eq!(report.intrinsic(), Some(&native));
        assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
        assert_eq!(report.context_author(), context);
        assert_eq!(report.textarea_wrap_cols_context(), None);
    }
    let cols = raw(Attr::Cols, "40");
    let complete = attributes(vec![Wrap::Hard.into_attribute(), cols.clone()]);
    let report = companion(&complete);
    assert_hard(&report);
    assert_eq!(
        report.context_author(),
        HtmlAttributeContextAuthorState::Conforming
    );
    assert_eq!(report.textarea_wrap_cols_context(), Some(Some(&cols)));
}

#[test]
fn ascii_case_keywords_keep_exact_raw_spelling_and_separate_intrinsic_state() {
    for (spelling, state, canonical, context) in [
        (
            "soft",
            Wrap::Soft,
            "soft",
            HtmlAttributeContextAuthorState::NotRequired,
        ),
        (
            "SOFT",
            Wrap::Soft,
            "soft",
            HtmlAttributeContextAuthorState::NotRequired,
        ),
        (
            "sOfT",
            Wrap::Soft,
            "soft",
            HtmlAttributeContextAuthorState::NotRequired,
        ),
        (
            "hard",
            Wrap::Hard,
            "hard",
            HtmlAttributeContextAuthorState::Unchecked,
        ),
        (
            "HARD",
            Wrap::Hard,
            "hard",
            HtmlAttributeContextAuthorState::Unchecked,
        ),
        (
            "hArD",
            Wrap::Hard,
            "hard",
            HtmlAttributeContextAuthorState::Unchecked,
        ),
    ] {
        let attribute = raw(Attr::Wrap, spelling);
        let report = content(Some(&attribute));
        assert_eq!(report.raw_attribute(), Some(&attribute));
        assert_eq!(report.raw(), Some(&DocumentString::from(spelling)));
        assert_eq!(report.intrinsic(), Some(&state));
        assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
        assert_eq!(
            report.recognition(),
            HtmlAttributeRecognition::Keyword {
                matched: canonical,
                canonical
            }
        );
        assert_eq!(report.context_author(), context);
        assert_eq!(report.textarea_wrap_cols_context(), None);
    }
}

#[test]
fn missing_and_empty_have_different_authorship_despite_the_same_soft_state() {
    let report = content(None);
    assert_eq!(report.raw_attribute(), None);
    assert_eq!(report.observation(), HtmlAttributeObservation::Missing);
    assert_eq!(report.recognition(), HtmlAttributeRecognition::Missing);
    assert_eq!(report.author(), HtmlAttributeAuthorState::Missing);
    assert_eq!(report.intrinsic(), Some(&Wrap::Soft));
    assert_eq!(
        report.context_author(),
        HtmlAttributeContextAuthorState::NotRequired
    );
    assert_eq!(report.textarea_wrap_cols_context(), None);
    let empty = raw(Attr::Wrap, "");
    let report = content(Some(&empty));
    assert_eq!(report.raw_attribute(), Some(&empty));
    assert_eq!(report.observation(), HtmlAttributeObservation::PresentEmpty);
    assert_eq!(report.recognition(), HtmlAttributeRecognition::Empty);
    assert_eq!(
        report.author(),
        HtmlAttributeAuthorState::Nonconforming(HtmlAttributeConformanceError::UnrecognizedKeyword)
    );
    assert_eq!(report.intrinsic(), Some(&Wrap::Soft));
    assert_eq!(
        report.context_author(),
        HtmlAttributeContextAuthorState::NotRequired
    );
    assert_eq!(report.textarea_wrap_cols_context(), None);
}

#[test]
fn no_trimming_unicode_folding_or_surrogate_replacement_makes_a_keyword() {
    for value in [
        DocumentString::from("unknown"),
        DocumentString::from(" hard"),
        DocumentString::from("hard "),
        DocumentString::from("\tsoft"),
        DocumentString::from("soft\n"),
        DocumentString::from("ſoft"),
        DocumentString::from("ｈａｒｄ"),
        DocumentString::from_utf16(vec![0]),
        DocumentString::from_utf16(vec![0xD800]),
        DocumentString::from_utf16(vec![0xDC00]),
        DocumentString::from_utf16(vec![0xD83D, 0xDE00]),
        DocumentString::from_utf16(vec![0x68, 0x61, 0x72, 0x64, 0xD800]),
    ] {
        let raw = raw(Attr::Wrap, value.clone());
        let report = content(Some(&raw));
        assert_eq!(report.raw_attribute(), Some(&raw));
        assert_eq!(report.raw(), Some(&value));
        assert_eq!(report.recognition(), HtmlAttributeRecognition::Unrecognized);
        assert_eq!(
            report.author(),
            HtmlAttributeAuthorState::Nonconforming(
                HtmlAttributeConformanceError::UnrecognizedKeyword
            )
        );
        assert_eq!(report.intrinsic(), Some(&Wrap::Soft));
        assert_eq!(
            report.context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
    }
}

#[test]
fn hard_with_known_missing_cols_preserves_hard_and_reports_only_the_presence_violation() {
    let wrap = raw(Attr::Wrap, "HaRd");
    let unrequested = content(Some(&wrap));
    assert_hard(&unrequested);
    assert_eq!(
        unrequested.context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
    assert_eq!(unrequested.textarea_wrap_cols_context(), None);
    let complete = attributes(vec![wrap.clone()]);
    let report = companion(&complete);
    assert_hard(&report);
    assert_eq!(report.raw_attribute(), Some(&wrap));
    assert_eq!(report.textarea_wrap_cols_context(), Some(None));
    assert_eq!(
        report.context_author(),
        HtmlAttributeContextAuthorState::Nonconforming(
            HtmlAttributeContextConformanceError::TextareaHardWrapRequiresCols
        )
    );
}

#[test]
fn hard_requires_presence_even_when_the_present_cols_text_has_no_valid_numeric_value() {
    // The wrap sentence says specified; numeric failure belongs only to textarea_cols.
    for (value, origin) in [
        (
            DocumentString::from(""),
            HtmlNumericAttributeValueOrigin::ParseFailureDefault,
        ),
        (
            DocumentString::from("garbage"),
            HtmlNumericAttributeValueOrigin::ParseFailureDefault,
        ),
        (
            DocumentString::from("0"),
            HtmlNumericAttributeValueOrigin::ZeroDefault,
        ),
        (
            DocumentString::from("-0"),
            HtmlNumericAttributeValueOrigin::ZeroDefault,
        ),
        (
            DocumentString::from("-2"),
            HtmlNumericAttributeValueOrigin::ParseFailureDefault,
        ),
        (
            DocumentString::from("+2tail"),
            HtmlNumericAttributeValueOrigin::Recovered,
        ),
        (
            DocumentString::from_utf16(vec![0xD800, 0xDC00]),
            HtmlNumericAttributeValueOrigin::ParseFailureDefault,
        ),
    ] {
        let cols = raw(Attr::Cols, value.clone());
        let complete = attributes(vec![Wrap::Hard.into_attribute(), cols.clone()]);
        let report = companion(&complete);
        assert_hard(&report);
        assert_eq!(report.textarea_wrap_cols_context(), Some(Some(&cols)));
        assert_eq!(
            report.context_author(),
            HtmlAttributeContextAuthorState::Conforming
        );
        let numeric = HtmlValueInterpreter::new(8192)
            .textarea_cols(&Element::Textarea.qualified_name(), Some(&cols))
            .unwrap();
        assert_eq!(numeric.raw(), Some(&value));
        assert!(matches!(
            numeric.author(),
            HtmlNumericAttributeAuthorState::Nonconforming(_)
        ));
        assert_eq!(numeric.origin(), Some(origin));
        let expected = if origin == HtmlNumericAttributeValueOrigin::Recovered {
            "2"
        } else {
            "20"
        };
        assert_eq!(numeric.intrinsic().unwrap().decimal_digits(), expected);
        if value == DocumentString::from("0") {
            assert!(matches!(
                numeric.common().unwrap().author(),
                HtmlValueAuthorState::Valid(_)
            ));
            assert_eq!(
                numeric.author(),
                HtmlNumericAttributeAuthorState::Nonconforming(
                    HtmlNumericAttributeConformanceError::ZeroMustBePositive
                )
            );
        } else {
            assert!(matches!(
                numeric.common().unwrap().author(),
                HtmlValueAuthorState::Invalid(_)
            ));
        }
    }
    let huge = "9".repeat(1024);
    let cols = raw(Attr::Cols, huge.clone());
    let complete = attributes(vec![Wrap::Hard.into_attribute(), cols.clone()]);
    let report = companion(&complete);
    assert_hard(&report);
    assert_eq!(
        report.context_author(),
        HtmlAttributeContextAuthorState::Conforming
    );
    let numeric = HtmlValueInterpreter::new(8192)
        .textarea_cols(&Element::Textarea.qualified_name(), Some(&cols))
        .unwrap();
    assert_eq!(
        numeric.author(),
        HtmlNumericAttributeAuthorState::Conforming
    );
    assert_eq!(numeric.intrinsic().unwrap().decimal_digits(), huge);
    assert_eq!(
        numeric.origin(),
        Some(HtmlNumericAttributeValueOrigin::Recovered)
    );
}

#[test]
fn soft_missing_and_invalid_wrap_do_not_select_the_cols_requirement() {
    for wrap in [
        None,
        Some(raw(Attr::Wrap, "soft")),
        Some(raw(Attr::Wrap, "")),
        Some(raw(Attr::Wrap, "hard ")),
    ] {
        for cols in [None, Some(raw(Attr::Cols, "garbage"))] {
            let complete = attributes(wrap.iter().chain(cols.iter()).cloned().collect());
            let report = companion(&complete);
            assert_eq!(report.intrinsic(), Some(&Wrap::Soft));
            assert_eq!(
                report.context_author(),
                HtmlAttributeContextAuthorState::NotRequired
            );
            assert_eq!(report.textarea_wrap_cols_context(), Some(cols.as_ref()));
            assert_eq!(report.raw_attribute(), wrap.as_ref());
        }
    }
}

#[test]
fn exact_expanded_names_and_prefixes_retain_source_and_companion_records() {
    let element = QualifiedName::new(Namespace::html(), Some("h".into()), "textarea");
    let wrap = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("w".into()), "wrap"),
        "HARD",
    );
    let cols = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("c".into()), "cols"),
        DocumentString::from_utf16(vec![0xD800, 0]),
    );
    let complete = attributes(vec![
        cols.clone(),
        wrap.clone(),
        raw(Attr::Name, "unrelated"),
    ]);
    let report = HtmlValueInterpreter::new(64)
        .textarea_wrap_with_cols(&element, &complete)
        .unwrap();
    assert_hard(&report);
    assert_eq!(report.element(), &element);
    assert_eq!(report.raw_attribute(), Some(&wrap));
    assert_eq!(report.textarea_wrap_cols_context(), Some(Some(&cols)));
    assert_eq!(report.attribute(), Attr::Wrap);
    assert_eq!(report.inventory_rows(), &[179]);
    assert_eq!(report.source_clause(), SOURCE);
    assert_eq!(report.context_source_clause(), Some(SOURCE));
    assert_eq!(report.vocabulary_source_clause(), None);
    assert_eq!(report.script_context(), None);
    assert_eq!(report.hyperlink_context(), None);
    assert_eq!(report.input_state(), None);
    assert_eq!(report.button_submitter_context(), None);
    assert_eq!(
        report.context_author(),
        HtmlAttributeContextAuthorState::Conforming
    );
    assert_eq!(content(None).context_source_clause(), Some(SOURCE));
}

#[test]
fn namespaced_and_uppercase_cols_do_not_satisfy_html_cols_presence() {
    for name in [
        QualifiedName::new(Namespace::none(), None, "COLS"),
        QualifiedName::new(Namespace::svg(), Some("s".into()), "cols"),
        QualifiedName::new(Namespace::html(), None, "cols"),
    ] {
        let unrelated = Attribute::new(name, "40");
        let complete = attributes(vec![Wrap::Hard.into_attribute(), unrelated]);
        let report = companion(&complete);
        assert_hard(&report);
        assert_eq!(report.textarea_wrap_cols_context(), Some(None));
        assert_eq!(
            report.context_author(),
            HtmlAttributeContextAuthorState::Nonconforming(
                HtmlAttributeContextConformanceError::TextareaHardWrapRequiresCols
            )
        );
    }
}

#[test]
fn non_html_wrap_slots_are_missing_in_the_collection_but_unapplied_when_directly_supplied() {
    for name in [
        QualifiedName::new(Namespace::none(), None, "WRAP"),
        QualifiedName::new(Namespace::svg(), Some("s".into()), "wrap"),
        Attr::Name.qualified_name(),
    ] {
        let unrelated = Attribute::new(name, "hard");
        let direct = content(Some(&unrelated));
        assert_eq!(
            direct.applicability(),
            HtmlAttributeApplicability::DifferentAttribute
        );
        assert_eq!(direct.recognition(), HtmlAttributeRecognition::Unapplied);
        assert_eq!(direct.author(), HtmlAttributeAuthorState::Unapplied);
        assert_eq!(direct.intrinsic(), None);
        assert_eq!(direct.textarea_wrap_cols_context(), None);
        let complete = attributes(vec![unrelated, raw(Attr::Cols, "garbage")]);
        let report = companion(&complete);
        assert_eq!(
            report.applicability(),
            HtmlAttributeApplicability::Applicable
        );
        assert_eq!(report.observation(), HtmlAttributeObservation::Missing);
        assert_eq!(report.recognition(), HtmlAttributeRecognition::Missing);
        assert_eq!(report.author(), HtmlAttributeAuthorState::Missing);
        assert_eq!(report.intrinsic(), Some(&Wrap::Soft));
        assert_eq!(
            report.context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
    }
}

#[test]
fn wrong_elements_and_namespaces_preserve_unapplied_wrap_and_observed_cols() {
    let wrap = raw(Attr::Wrap, "hard");
    let cols = raw(Attr::Cols, "garbage");
    let complete = attributes(vec![wrap.clone(), cols.clone()]);
    for (element, applicability) in [
        (
            Element::Input.qualified_name(),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            QualifiedName::new(Namespace::html(), None, "x-widget"),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            QualifiedName::new(Namespace::html(), None, "TEXTAREA"),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            QualifiedName::new(Namespace::svg(), None, "textarea"),
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
        (
            QualifiedName::new(Namespace::mathml(), None, "textarea"),
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
        (
            QualifiedName::new(Namespace::none(), None, "textarea"),
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
    ] {
        for report in [
            HtmlValueInterpreter::new(64)
                .textarea_wrap(&element, Some(&wrap))
                .unwrap(),
            HtmlValueInterpreter::new(64)
                .textarea_wrap_with_cols(&element, &complete)
                .unwrap(),
        ] {
            assert_eq!(report.element(), &element);
            assert_eq!(report.applicability(), applicability);
            assert_eq!(report.raw_attribute(), Some(&wrap));
            assert_eq!(report.intrinsic(), None);
            assert_eq!(report.author(), HtmlAttributeAuthorState::Unapplied);
            assert_eq!(report.recognition(), HtmlAttributeRecognition::Unapplied);
            assert_eq!(
                report.context_author(),
                HtmlAttributeContextAuthorState::Unapplied
            );
        }
        let report = HtmlValueInterpreter::new(64)
            .textarea_wrap_with_cols(&element, &complete)
            .unwrap();
        assert_eq!(report.textarea_wrap_cols_context(), Some(Some(&cols)));
    }
}

#[test]
fn both_selected_values_are_admitted_before_role_or_content_and_count_original_units() {
    for units in [vec![0xD800], vec![0xD83D, 0xDE00], vec![0xD800, 0, 0xDC00]] {
        let value = DocumentString::from_utf16(units.clone());
        for element in [
            Element::Textarea.qualified_name(),
            Element::Input.qualified_name(),
            QualifiedName::new(Namespace::svg(), None, "textarea"),
        ] {
            let raw = raw(Attr::Wrap, value.clone());
            let error = HtmlValueInterpreter::new(units.len() - 1)
                .textarea_wrap(&element, Some(&raw))
                .unwrap_err();
            assert_eq!(error.capacity(), units.len() - 1);
            assert_eq!(error.required(), units.len());
            let report = HtmlValueInterpreter::new(units.len())
                .textarea_wrap(&element, Some(&raw))
                .unwrap();
            assert_eq!(report.raw().unwrap().code_units(), units);
            assert_eq!(report.input_code_units(), units.len());
        }
    }
    // A short Soft wrap does not bypass admission of selected malformed cols.
    for element in [
        Element::Textarea.qualified_name(),
        Element::Input.qualified_name(),
        QualifiedName::new(Namespace::svg(), None, "textarea"),
    ] {
        let cols = raw(Attr::Cols, DocumentString::from_utf16(vec![0xD800; 5]));
        for spelling in ["soft", "hard", ""] {
            let wrap = raw(Attr::Wrap, spelling);
            let complete = attributes(vec![wrap.clone(), cols.clone()]);
            let error = HtmlValueInterpreter::new(4)
                .textarea_wrap_with_cols(&element, &complete)
                .unwrap_err();
            assert_eq!(error.capacity(), 4);
            assert_eq!(error.required(), 5);
            let report = HtmlValueInterpreter::new(5)
                .textarea_wrap_with_cols(&element, &complete)
                .unwrap();
            assert_eq!(report.raw_attribute(), Some(&wrap));
            assert_eq!(report.textarea_wrap_cols_context(), Some(Some(&cols)));
        }
    }
    let long_wrap = raw(Attr::Wrap, "longer");
    let long_cols = raw(Attr::Cols, "even-longer");
    let both = attributes(vec![long_cols, long_wrap]);
    let error = HtmlValueInterpreter::new(4)
        .textarea_wrap_with_cols(&Element::Input.qualified_name(), &both)
        .unwrap_err();
    assert_eq!(error.required(), 6); // wrap is admitted first, not collection order.
}

#[test]
fn zero_caps_allow_missing_and_empty_without_becoming_a_collection_cap() {
    let interpreter = HtmlValueInterpreter::new(0);
    let element = Element::Textarea.qualified_name();
    let missing = interpreter
        .textarea_wrap_with_cols(&element, &OrderedAttributes::default())
        .unwrap();
    assert_eq!(missing.author(), HtmlAttributeAuthorState::Missing);
    assert_eq!(missing.intrinsic(), Some(&Wrap::Soft));
    assert_eq!(missing.textarea_wrap_cols_context(), Some(None));
    let empty_cols = raw(Attr::Cols, "");
    let selected_empty = attributes(vec![raw(Attr::Wrap, ""), empty_cols.clone()]);
    let empty = interpreter
        .textarea_wrap_with_cols(&element, &selected_empty)
        .unwrap();
    assert_eq!(empty.intrinsic(), Some(&Wrap::Soft));
    assert_eq!(empty.recognition(), HtmlAttributeRecognition::Empty);
    assert_eq!(empty.textarea_wrap_cols_context(), Some(Some(&empty_cols)));
    let unselected = attributes(vec![Attribute::new(
        QualifiedName::new(Namespace::svg(), None, "cols"),
        "x".repeat(1024),
    )]);
    assert_eq!(
        interpreter
            .textarea_wrap_with_cols(&element, &unselected)
            .unwrap()
            .intrinsic(),
        Some(&Wrap::Soft)
    );
    let error = interpreter
        .textarea_wrap(&element, Some(&Wrap::Hard.into_attribute()))
        .unwrap_err();
    assert_eq!(error.capacity(), 0);
    assert_eq!(error.required(), 4);
}

#[test]
fn retained_context_and_raw_backing_survive_collection_drop_and_later_calls() {
    let wrap = raw(Attr::Wrap, "HaRd");
    let cols = raw(Attr::Cols, DocumentString::from_utf16(vec![0xD800; 8192]));
    let complete = attributes(vec![cols.clone(), wrap.clone()]);
    let report = companion(&complete);
    let retained = report.clone();
    assert_hard(&report);
    assert_eq!(
        report.context_author(),
        HtmlAttributeContextAuthorState::Conforming
    );
    assert_eq!(report.textarea_wrap_cols_context(), Some(Some(&cols)));
    drop(complete);
    let subsequent = companion(&attributes(vec![Wrap::Hard.into_attribute()]));
    assert_eq!(
        subsequent.context_author(),
        HtmlAttributeContextAuthorState::Nonconforming(
            HtmlAttributeContextConformanceError::TextareaHardWrapRequiresCols
        )
    );
    assert_eq!(report, retained);
    assert_eq!(report.raw_attribute(), Some(&wrap));
    assert_eq!(report.textarea_wrap_cols_context(), Some(Some(&cols)));
}
