use surgeist_html::{
    Attribute, DocumentString, HtmlAttributeApplicability, HtmlAttributeAuthorState,
    HtmlAttributeConformanceError, HtmlAttributeContextAuthorState, HtmlAttributeObservation,
    HtmlAttributeRecognition, HtmlAttributeReport, HtmlCustomElementName,
    HtmlCustomElementNameError, HtmlValueInterpreter, KnownHtmlAttribute, KnownHtmlElement,
    Namespace, QualifiedName,
};
use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::{
    HtmlCaptureBuilder, HtmlCaptureError, HtmlCaptureLimits, HtmlCustomElementDefinition,
};

fn raw(value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(KnownHtmlAttribute::Is.qualified_name(), value)
}

fn read(raw: Option<&Attribute>) -> HtmlAttributeReport<HtmlCustomElementName> {
    HtmlValueInterpreter::new(4096)
        .is_attribute(&KnownHtmlElement::Button.qualified_name(), raw)
        .unwrap()
}

fn conforming(value: &DocumentString) {
    let attribute = raw(value.clone());
    let report = read(Some(&attribute));
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
    assert_eq!(report.raw_attribute(), Some(&attribute));
    assert_eq!(report.raw(), Some(value));
    assert_eq!(report.intrinsic().unwrap().value(), value);
    assert_eq!(
        HtmlCustomElementName::new(value.clone()).unwrap().value(),
        value
    );
}

fn rejected(value: &DocumentString, error: HtmlCustomElementNameError) {
    assert_eq!(HtmlCustomElementName::new(value.clone()), Err(error));
    let attribute = raw(value.clone());
    let report = read(Some(&attribute));
    assert_eq!(report.raw_attribute(), Some(&attribute));
    assert_eq!(report.raw(), Some(value));
    assert_eq!(
        report.applicability(),
        HtmlAttributeApplicability::Applicable
    );
    assert_eq!(report.intrinsic(), None);
    assert_eq!(
        report.context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
    assert_eq!(
        report.author(),
        HtmlAttributeAuthorState::Nonconforming(
            HtmlAttributeConformanceError::InvalidCustomElementName(error)
        )
    );
}

#[test]
fn valid_native_names_preserve_source_permitted_punctuation_and_non_ascii_text() {
    // HTML §4.13.3 + DOM ASCII-alpha-first local-name branch, not old PCENChar.
    for text in [
        "x-a",
        "a-",
        "math-α",
        "emotion-😍",
        "x-<",
        "x-=",
        "x-\"",
        "x-'",
        "x-:",
        "x-_",
        "x-.",
        "x-+1",
        "x-Ä",
        "x-Α",
        "x-\u{a0}",
        "x-\u{2003}",
        "x-\u{fdd0}",
        "x-\u{ffff}",
        "x-\u{10ffff}",
        "x-é",
        "x-e\u{301}",
    ] {
        conforming(&text.into());
    }
    assert_ne!(
        HtmlCustomElementName::new("x-é").unwrap(),
        HtmlCustomElementName::new("x-e\u{301}").unwrap()
    );
}

#[test]
fn supplementary_and_isolated_surrogates_survive_checked_construction_and_reports() {
    for units in [
        vec![0x78, 0x2d, 0xd800],
        vec![0x78, 0x2d, 0xdc00],
        vec![0x78, 0x2d, 0xd83d, 0xde0d],
        vec![0x78, 0x2d, 0xdc00, 0xd800, 0xfffe, 0x3c],
    ] {
        let value = DocumentString::from_utf16(units.clone());
        conforming(&value);
        let native = HtmlCustomElementName::new(value.clone()).unwrap();
        assert_eq!(native.value().code_units(), units);
        let name = native.clone().into_qualified_name();
        assert_eq!(name.namespace(), &Namespace::html());
        assert_eq!(name.prefix(), None);
        assert_eq!(name.local_name().code_units(), units);
        let emitted = native.into_is_attribute();
        assert_eq!(emitted.name(), &KnownHtmlAttribute::Is.qualified_name());
        assert_eq!(emitted.value().code_units(), units);
        assert_eq!(read(Some(&emitted)).raw(), Some(&value));
    }
}

#[test]
fn empty_or_nonlowercase_initial_code_points_do_not_become_valid_names() {
    rejected(&"".into(), HtmlCustomElementNameError::Empty);
    for text in [
        "-x", "+x-a", "1-x", "_x-a", ":x-a", "X-a", " x-a", "é-x", "😍-x",
    ] {
        rejected(
            &text.into(),
            HtmlCustomElementNameError::FirstCodePointNotLowercaseAscii,
        );
    }
    for first in [0xd800, 0xdc00] {
        rejected(
            &DocumentString::from_utf16(vec![first, 0x2d, 0x78]),
            HtmlCustomElementNameError::FirstCodePointNotLowercaseAscii,
        );
    }
}

#[test]
fn a_hyphen_is_required_without_guessing_registration() {
    for text in ["x", "widget", "a_1", "xα", "x😍"] {
        rejected(&text.into(), HtmlCustomElementNameError::MissingHyphen);
    }
    let valid = raw("x-unregistered");
    let report = read(Some(&valid));
    assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
    assert_eq!(
        report.context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
}

#[test]
fn each_reserved_name_is_rejected_exactly() {
    // The eight names are literal independent expectations from the selected HTML list.
    for text in [
        "annotation-xml",
        "color-profile",
        "font-face",
        "font-face-src",
        "font-face-uri",
        "font-face-format",
        "font-face-name",
        "missing-glyph",
    ] {
        rejected(&text.into(), HtmlCustomElementNameError::ReservedName);
    }
    for text in ["x-annotation-xml", "font-face-more", "missing-glyph-x"] {
        conforming(&text.into());
    }
}

#[test]
fn forbidden_ascii_units_are_rejected_without_trimming_or_case_folding() {
    for unit in [0, 9, 10, 12, 13, 32, 47, 62, 0x41, 0x5a] {
        rejected(
            &DocumentString::from_utf16(vec![0x78, 0x2d, unit, 0x79]),
            HtmlCustomElementNameError::ForbiddenCodeUnit {
                code_unit_offset: 2,
                code_unit: unit,
            },
        );
    }
    rejected(
        &"x-a ".into(),
        HtmlCustomElementNameError::ForbiddenCodeUnit {
            code_unit_offset: 3,
            code_unit: 32,
        },
    );
    rejected(
        &"x-A".into(),
        HtmlCustomElementNameError::ForbiddenCodeUnit {
            code_unit_offset: 2,
            code_unit: 0x41,
        },
    );
    assert_eq!(
        HtmlCustomElementName::new("X-a"),
        Err(HtmlCustomElementNameError::FirstCodePointNotLowercaseAscii)
    );
}

#[test]
fn forbidden_offsets_count_utf16_units_including_supplementary_pairs() {
    for (units, offset) in [
        (vec![0x78, 0x2d, 0xd800, 0x20], 3),
        (vec![0x78, 0x2d, 0xdc00, 0x09], 3),
        (vec![0x78, 0x2d, 0xd83d, 0xde00, 0x2f], 4),
    ] {
        let forbidden = *units.last().unwrap();
        rejected(
            &DocumentString::from_utf16(units),
            HtmlCustomElementNameError::ForbiddenCodeUnit {
                code_unit_offset: offset,
                code_unit: forbidden,
            },
        );
    }
}

#[test]
fn native_emission_preserves_checked_value_with_intentional_qualified_names() {
    let value = DocumentString::from("x-é");
    let native = HtmlCustomElementName::new(value.clone()).unwrap();
    let name = native.clone().into_qualified_name();
    assert_eq!(
        name,
        QualifiedName::new(Namespace::html(), None, value.clone())
    );
    let emitted = native.into_is_attribute();
    assert_eq!(emitted.name().namespace(), &Namespace::none());
    assert_eq!(emitted.name().local_name(), &DocumentString::from("is"));
    assert_eq!(emitted.name().prefix(), None);
    assert_eq!(emitted.value(), &value);
    assert_eq!(read(Some(&emitted)).intrinsic().unwrap().value(), &value);
}

#[test]
fn absent_is_differs_from_empty_and_has_no_default_checked_name() {
    let missing = read(None);
    assert_eq!(missing.observation(), HtmlAttributeObservation::Missing);
    assert_eq!(missing.recognition(), HtmlAttributeRecognition::Missing);
    assert_eq!(missing.author(), HtmlAttributeAuthorState::Missing);
    assert_eq!(
        missing.context_author(),
        HtmlAttributeContextAuthorState::NotRequired
    );
    assert_eq!(missing.raw(), None);
    assert_eq!(missing.intrinsic(), None);
    let empty = raw("");
    let report = read(Some(&empty));
    assert_eq!(report.observation(), HtmlAttributeObservation::PresentEmpty);
    assert_eq!(report.recognition(), HtmlAttributeRecognition::Empty);
    rejected(&"".into(), HtmlCustomElementNameError::Empty);
}

#[test]
fn local_content_reports_leave_actual_custom_element_context_unchecked() {
    // A syntactically custom name is not proof of autonomous/customized classification.
    for element in [
        KnownHtmlElement::Button.qualified_name(),
        KnownHtmlElement::Div.qualified_name(),
        QualifiedName::new(Namespace::html(), None, "x-autonomous"),
        QualifiedName::new(Namespace::html(), Some("h".into()), "unknown"),
    ] {
        for text in ["x-widget", "", "x-widget "] {
            let attribute = raw(text);
            let report = HtmlValueInterpreter::new(64)
                .is_attribute(&element, Some(&attribute))
                .unwrap();
            assert_eq!(report.element(), &element);
            assert_eq!(
                report.applicability(),
                HtmlAttributeApplicability::Applicable
            );
            assert_eq!(
                report.context_author(),
                HtmlAttributeContextAuthorState::Unchecked
            );
            assert_eq!(report.raw_attribute(), Some(&attribute));
        }
    }
}

#[test]
fn source_and_prefix_metadata_follow_expanded_names_without_rewriting_raw_records() {
    let element = QualifiedName::new(Namespace::html(), Some("h".into()), "button");
    let attribute = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("p".into()), "is"),
        "x-a",
    );
    let report = HtmlValueInterpreter::new(64)
        .is_attribute(&element, Some(&attribute))
        .unwrap();
    assert_eq!(report.element(), &element);
    assert_eq!(report.raw_attribute(), Some(&attribute));
    assert_eq!(
        report.intrinsic().unwrap().value(),
        &DocumentString::from("x-a")
    );
    assert_eq!(report.attribute(), KnownHtmlAttribute::Is);
    assert_eq!(report.inventory_rows(), &[78]);
    assert_eq!(
        report.source_clause(),
        "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-is"
    );
    assert_eq!(report.vocabulary_source_clause(), None);
    assert_eq!(report.script_context(), None);
    assert_eq!(report.custom_control_context(), None);
    let emitted = report.intrinsic().unwrap().clone().into_is_attribute();
    assert_eq!(emitted.name().prefix(), None);
    assert_eq!(report.raw_attribute(), Some(&attribute));
}

#[test]
fn wrong_expanded_role_retains_raw_unapplied_without_a_default() {
    let invalid = raw("");
    let button = KnownHtmlElement::Button.qualified_name();
    let uppercase = Attribute::new(QualifiedName::new(Namespace::none(), None, "IS"), "");
    let different = Attribute::new(KnownHtmlAttribute::Id.qualified_name(), "");
    let foreign = Attribute::new(QualifiedName::new(Namespace::svg(), None, "is"), "");
    for (element, attribute, expected) in [
        (
            QualifiedName::new(Namespace::svg(), None, "button"),
            &invalid,
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
        (
            QualifiedName::new(Namespace::mathml(), None, "button"),
            &invalid,
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
        (
            QualifiedName::new(Namespace::none(), None, "button"),
            &invalid,
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
        (
            button.clone(),
            &uppercase,
            HtmlAttributeApplicability::DifferentAttribute,
        ),
        (
            button.clone(),
            &different,
            HtmlAttributeApplicability::DifferentAttribute,
        ),
        (
            button,
            &foreign,
            HtmlAttributeApplicability::DifferentAttribute,
        ),
    ] {
        let report = HtmlValueInterpreter::new(64)
            .is_attribute(&element, Some(attribute))
            .unwrap();
        assert_eq!(report.applicability(), expected);
        assert_eq!(report.observation(), HtmlAttributeObservation::PresentEmpty);
        assert_eq!(report.recognition(), HtmlAttributeRecognition::Unapplied);
        assert_eq!(report.author(), HtmlAttributeAuthorState::Unapplied);
        assert_eq!(
            report.context_author(),
            HtmlAttributeContextAuthorState::Unapplied
        );
        assert_eq!(report.raw_attribute(), Some(attribute));
        assert_eq!(report.intrinsic(), None);
    }
    let absent = HtmlValueInterpreter::new(0)
        .is_attribute(
            &QualifiedName::new(Namespace::svg(), None, "button"),
            None::<&surgeist_html::Attribute>,
        )
        .unwrap();
    assert_eq!(
        absent.applicability(),
        HtmlAttributeApplicability::ForeignElementNamespace
    );
    assert_eq!(absent.observation(), HtmlAttributeObservation::Missing);
    assert_eq!(absent.author(), HtmlAttributeAuthorState::Unapplied);
}

#[test]
fn original_raw_admission_precedes_local_invalidity_and_wrong_role() {
    for units in [
        vec![0x78, 0x2d, 0xd800],
        vec![0x78, 0x2d, 0xd83d, 0xde00],
        vec![0x78, 0x2d, 0x20],
        vec![0x58, 0x2d, 0x78],
    ] {
        let value = DocumentString::from_utf16(units.clone());
        for (element, name) in [
            (
                KnownHtmlElement::Button.qualified_name(),
                KnownHtmlAttribute::Is.qualified_name(),
            ),
            (
                KnownHtmlElement::Button.qualified_name(),
                KnownHtmlAttribute::Id.qualified_name(),
            ),
            (
                QualifiedName::new(Namespace::svg(), None, "button"),
                KnownHtmlAttribute::Is.qualified_name(),
            ),
        ] {
            let attribute = Attribute::new(name, value.clone());
            let error = HtmlValueInterpreter::new(units.len() - 1)
                .is_attribute(&element, Some(&attribute))
                .unwrap_err();
            assert_eq!(error.required(), units.len());
            assert_eq!(error.capacity(), units.len() - 1);
            let admitted = HtmlValueInterpreter::new(units.len())
                .is_attribute(&element, Some(&attribute))
                .unwrap();
            assert_eq!(admitted.raw().unwrap().code_units(), units);
            assert_eq!(admitted.raw_attribute(), Some(&attribute));
        }
    }
    let zero = HtmlValueInterpreter::new(0);
    assert_eq!(
        zero.is_attribute(
            &KnownHtmlElement::Button.qualified_name(),
            None::<&surgeist_html::Attribute>
        )
        .unwrap()
        .author(),
        HtmlAttributeAuthorState::Missing
    );
    let empty = raw("");
    assert_eq!(
        zero.is_attribute(&KnownHtmlElement::Button.qualified_name(), Some(&empty))
            .unwrap()
            .author(),
        HtmlAttributeAuthorState::Nonconforming(
            HtmlAttributeConformanceError::InvalidCustomElementName(
                HtmlCustomElementNameError::Empty
            )
        )
    );
}

#[test]
fn long_names_and_old_reports_survive_later_invalid_input_without_truncation() {
    let value = DocumentString::from(format!("x-{}", "a".repeat(4094)));
    let attribute = raw(value.clone());
    let report = read(Some(&attribute));
    assert_eq!(report.intrinsic().unwrap().value(), &value);
    let retained = report.clone();
    drop(attribute);
    let invalid = read(Some(&raw("x-A")));
    assert_eq!(invalid.intrinsic(), None);
    assert_eq!(report, retained);
    let error = HtmlValueInterpreter::new(4095)
        .is_attribute(
            &KnownHtmlElement::Button.qualified_name(),
            Some(&raw(value.clone())),
        )
        .unwrap_err();
    assert_eq!(error.required(), 4096);
    assert_eq!(error.capacity(), 4095);
    assert_eq!(report.intrinsic().unwrap().value(), &value);
}

#[test]
fn capture_definition_admission_preserves_the_shared_name_predicate_and_atomic_usage() {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 100, 100, 1000, 100, 100, 100),
        HtmlHostFactsRevision::fresh(),
    );
    for text in [
        "annotation-xml",
        "color-profile",
        "font-face",
        "font-face-src",
        "font-face-uri",
        "font-face-format",
        "font-face-name",
        "missing-glyph",
        "x",
        "X-a",
        "x-A",
        "x-a ",
        "x-/",
        "x->",
        "x-\0",
    ] {
        let before = builder.usage();
        assert_eq!(
            builder.add_registry(Some(vec![HtmlCustomElementDefinition::new(
                text.into(),
                "div".into(),
                false,
                false
            )])),
            Err(HtmlCaptureError::InvalidDefinitionName)
        );
        assert_eq!(builder.usage(), before);
    }
    for units in [
        vec![0x78, 0x2d, 0x3c, 0xd800],
        vec![0x78, 0x2d, 0x3d, 0x22, 0xffff],
    ] {
        let value = DocumentString::from_utf16(units);
        assert!(HtmlCustomElementName::new(value.clone()).is_ok());
        builder
            .add_registry(Some(vec![HtmlCustomElementDefinition::new(
                value,
                "div".into(),
                false,
                false,
            )]))
            .unwrap();
    }
}
