use surgeist_html::*;

#[derive(Clone, Copy)]
enum Role {
    Form,
    Meta,
}
impl Role {
    fn element(self) -> QualifiedName {
        match self {
            Self::Form => KnownHtmlElement::Form,
            Self::Meta => KnownHtmlElement::Meta,
        }
        .qualified_name()
    }
    fn attribute(self) -> KnownHtmlAttribute {
        match self {
            Self::Form => KnownHtmlAttribute::AcceptCharset,
            Self::Meta => KnownHtmlAttribute::Charset,
        }
    }
    fn raw(self, value: impl Into<DocumentString>) -> Attribute {
        Attribute::new(self.attribute().qualified_name(), value)
    }
    fn read(
        self,
        cap: usize,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<HtmlUtf8EncodingLabel>, HtmlValueAdmissionError> {
        let interpreter = HtmlValueInterpreter::new(cap);
        match self {
            Self::Form => interpreter.form_accept_charset(element, raw),
            Self::Meta => interpreter.meta_charset(element, raw),
        }
    }
    fn present_context(self) -> HtmlAttributeContextAuthorState {
        match self {
            Self::Form => HtmlAttributeContextAuthorState::NotRequired,
            Self::Meta => HtmlAttributeContextAuthorState::Unchecked,
        }
    }
}

#[test]
fn every_ascii_case_position_matches_without_changing_the_accepted_spelling() {
    // Independently enumerate the three alphabetic positions of the required label.
    for spelling in [
        "utf-8", "Utf-8", "uTf-8", "utF-8", "UTf-8", "UtF-8", "uTF-8", "UTF-8",
    ] {
        let checked = HtmlUtf8EncodingLabel::new(spelling).unwrap();
        assert_eq!(checked.value(), &DocumentString::from(spelling));
        for role in [Role::Form, Role::Meta] {
            let raw = role.raw(spelling);
            let report = role.read(5, &role.element(), Some(&raw)).unwrap();
            assert_eq!(
                report.applicability(),
                HtmlAttributeApplicability::Applicable
            );
            assert_eq!(report.raw_attribute(), Some(&raw));
            assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
            assert_eq!(
                report.recognition(),
                HtmlAttributeRecognition::Keyword {
                    matched: "utf-8",
                    canonical: "utf-8",
                }
            );
            assert_eq!(report.intrinsic(), Some(&checked));
            assert_eq!(report.context_author(), role.present_context());
        }
    }
}

#[test]
fn absence_and_present_empty_remain_distinct_and_select_no_default_label() {
    for role in [Role::Form, Role::Meta] {
        let absent = role.read(0, &role.element(), None).unwrap();
        assert_eq!(absent.observation(), HtmlAttributeObservation::Missing);
        assert_eq!(absent.recognition(), HtmlAttributeRecognition::Missing);
        assert_eq!(absent.author(), HtmlAttributeAuthorState::Missing);
        assert_eq!(
            absent.context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
        assert_eq!(absent.raw(), None);
        assert_eq!(absent.intrinsic(), None);
        let raw = role.raw("");
        let empty = role.read(0, &role.element(), Some(&raw)).unwrap();
        assert_eq!(empty.observation(), HtmlAttributeObservation::PresentEmpty);
        assert_eq!(empty.recognition(), HtmlAttributeRecognition::Empty);
        assert_eq!(
            empty.author(),
            HtmlAttributeAuthorState::Nonconforming(
                HtmlAttributeConformanceError::InvalidUtf8EncodingLabel(HtmlUtf8EncodingLabelError),
            )
        );
        assert_eq!(empty.context_author(), role.present_context());
        assert_eq!(empty.raw_attribute(), Some(&raw));
        assert_eq!(empty.intrinsic(), None);
    }
}

#[test]
fn aliases_padding_lists_and_other_encodings_are_not_the_required_author_spelling() {
    for spelling in [
        "utf8",
        "UTF8",
        "unicode-1-1-utf-8",
        "windows-1252",
        "latin1",
        "UTF-16",
        " UTF-8",
        "UTF-8 ",
        "\tUTF-8",
        "UTF-8\n",
        "UTF-8\r",
        "UTF-8\u{c}",
        "UTF-8 UTF-8",
        "UTF-8,UTF-8",
        "windows-1252 UTF-8",
        "UTF-8\0",
        "\0UTF-8",
        "ＵＴＦ-8",
        "utf‐8",
        "utf-８",
        "utf-8\u{a0}",
    ] {
        assert_eq!(
            HtmlUtf8EncodingLabel::new(spelling),
            Err(HtmlUtf8EncodingLabelError)
        );
        for role in [Role::Form, Role::Meta] {
            let raw = role.raw(spelling);
            let report = role.read(128, &role.element(), Some(&raw)).unwrap();
            assert_eq!(
                report.observation(),
                HtmlAttributeObservation::PresentNonempty
            );
            assert_eq!(report.recognition(), HtmlAttributeRecognition::Unrecognized);
            assert_eq!(
                report.author(),
                HtmlAttributeAuthorState::Nonconforming(
                    HtmlAttributeConformanceError::InvalidUtf8EncodingLabel(
                        HtmlUtf8EncodingLabelError
                    ),
                )
            );
            assert_eq!(report.context_author(), role.present_context());
            assert_eq!(report.raw_attribute(), Some(&raw));
            assert_eq!(report.intrinsic(), None);
        }
    }
}

#[test]
fn invalid_utf16_is_retained_exactly_without_replacement_or_prefix_acceptance() {
    for units in [
        vec![0xD800],
        vec![0xDC00],
        vec![0xD83D, 0xDE00],
        vec![0x75, 0x74, 0x66, 0x2D, 0xD800],
        vec![0x75, 0x74, 0x66, 0x2D, 0x38, 0xD800],
        vec![0xD800, 0, 0xDC00, 0xD83D, 0xDE00],
    ] {
        let value = DocumentString::from_utf16(units.clone());
        assert_eq!(
            HtmlUtf8EncodingLabel::new(value.clone()),
            Err(HtmlUtf8EncodingLabelError)
        );
        for role in [Role::Form, Role::Meta] {
            let raw = role.raw(value.clone());
            let report = role.read(units.len(), &role.element(), Some(&raw)).unwrap();
            assert_eq!(report.raw().unwrap().code_units(), units);
            assert_eq!(report.raw_attribute(), Some(&raw));
            assert_eq!(report.recognition(), HtmlAttributeRecognition::Unrecognized);
            assert_eq!(report.intrinsic(), None);
            assert_eq!(report.context_author(), role.present_context());
        }
    }
}

#[test]
fn native_emission_preserves_mixed_case_and_uses_canonical_expanded_attribute_names() {
    let checked =
        HtmlUtf8EncodingLabel::new(DocumentString::from_utf16(vec![85, 116, 70, 45, 56])).unwrap();
    assert_eq!(checked.value().code_units(), &[85, 116, 70, 45, 56]);
    assert_ne!(checked, HtmlUtf8EncodingLabel::new("utf-8").unwrap());
    assert_eq!(
        checked.clone(),
        HtmlUtf8EncodingLabel::new("UtF-8").unwrap()
    );
    for (role, attribute) in [
        (Role::Form, checked.clone().into_accept_charset_attribute()),
        (Role::Meta, checked.into_charset_attribute()),
    ] {
        assert_eq!(attribute.name(), &role.attribute().qualified_name());
        assert_eq!(attribute.name().namespace(), &Namespace::none());
        assert_eq!(attribute.name().prefix(), None);
        assert_eq!(attribute.value().code_units(), &[85, 116, 70, 45, 56]);
        let report = role.read(5, &role.element(), Some(&attribute)).unwrap();
        assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
        assert_eq!(report.intrinsic().unwrap().value(), attribute.value());
    }
    let error: &dyn std::error::Error = &HtmlUtf8EncodingLabelError;
    assert!(error.source().is_none());
    assert_eq!(
        error.to_string(),
        "HTML requires an exact ASCII-case-insensitive UTF-8 label"
    );
}

#[test]
fn exact_source_rows_and_context_metadata_do_not_certify_document_or_submission_encoding() {
    for (role, row, local, anchor, context_source) in [
        (
            Role::Form,
            3,
            "accept-charset",
            "attr-form-accept-charset",
            None,
        ),
        (
            Role::Meta,
            19,
            "charset",
            "attr-meta-charset",
            Some("the-meta-element"),
        ),
    ] {
        let element = QualifiedName::new(
            Namespace::html(),
            Some("h".into()),
            role.element().local_name().clone(),
        );
        let attribute = Attribute::new(
            QualifiedName::new(Namespace::none(), Some("a".into()), local),
            "uTF-8",
        );
        let report = role.read(5, &element, Some(&attribute)).unwrap();
        assert_eq!(report.element(), &element);
        assert_eq!(report.raw_attribute(), Some(&attribute));
        assert_eq!(report.inventory_rows(), &[row]);
        assert_eq!(report.attribute(), role.attribute());
        assert_eq!(
            report.source_clause(),
            format!(
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#{anchor}"
            )
        );
        assert_eq!(report.context_source_clause().map(str::to_owned), context_source.map(|anchor| format!(
            "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#{anchor}")));
        assert_eq!(report.vocabulary_source_clause(), None);
        assert_eq!(report.script_context(), None);
        assert_eq!(report.hyperlink_context(), None);
        assert_eq!(report.context_author(), role.present_context());
    }
}

#[test]
fn wrong_roles_retain_raw_evidence_with_unapplied_states() {
    for role in [Role::Form, Role::Meta] {
        let expected = role.element();
        let raw = role.raw("UTF-8");
        let wrong = Attribute::new(KnownHtmlAttribute::Id.qualified_name(), "UTF-8");
        let foreign = Attribute::new(
            QualifiedName::new(Namespace::svg(), None, role.attribute().local_name()),
            "UTF-8",
        );
        let upper = Attribute::new(
            QualifiedName::new(
                Namespace::none(),
                None,
                role.attribute().local_name().to_ascii_uppercase(),
            ),
            "UTF-8",
        );
        for (element, attribute, applicability) in [
            (
                KnownHtmlElement::Div.qualified_name(),
                &raw,
                HtmlAttributeApplicability::DifferentElement,
            ),
            (
                QualifiedName::new(
                    Namespace::html(),
                    None,
                    expected
                        .local_name()
                        .to_utf8()
                        .unwrap()
                        .to_ascii_uppercase(),
                ),
                &raw,
                HtmlAttributeApplicability::DifferentElement,
            ),
            (
                QualifiedName::new(Namespace::none(), None, expected.local_name().clone()),
                &raw,
                HtmlAttributeApplicability::ForeignElementNamespace,
            ),
            (
                QualifiedName::new(
                    Namespace::svg(),
                    Some("s".into()),
                    expected.local_name().clone(),
                ),
                &raw,
                HtmlAttributeApplicability::ForeignElementNamespace,
            ),
            (
                expected.clone(),
                &wrong,
                HtmlAttributeApplicability::DifferentAttribute,
            ),
            (
                expected.clone(),
                &foreign,
                HtmlAttributeApplicability::DifferentAttribute,
            ),
            (
                expected,
                &upper,
                HtmlAttributeApplicability::DifferentAttribute,
            ),
        ] {
            let report = role.read(5, &element, Some(attribute)).unwrap();
            assert_eq!(report.applicability(), applicability);
            assert_eq!(report.raw_attribute(), Some(attribute));
            assert_eq!(report.element(), &element);
            assert_eq!(
                report.observation(),
                HtmlAttributeObservation::PresentNonempty
            );
            assert_eq!(report.recognition(), HtmlAttributeRecognition::Unapplied);
            assert_eq!(report.author(), HtmlAttributeAuthorState::Unapplied);
            assert_eq!(
                report.context_author(),
                HtmlAttributeContextAuthorState::Unapplied
            );
            assert_eq!(report.intrinsic(), None);
        }
    }
}

#[test]
fn entire_raw_cap_precedes_applicability_and_measures_utf16_units() {
    for role in [Role::Form, Role::Meta] {
        for units in [
            vec![0xD800],
            vec![0xD83D, 0xDE00],
            vec![0, 0xDC00],
            vec![85, 84, 70, 45, 56],
            vec![85, 84, 70, 45, 56, 0xD800],
        ] {
            let value = DocumentString::from_utf16(units.clone());
            for (element, name) in [
                (role.element(), role.attribute().qualified_name()),
                (
                    KnownHtmlElement::Div.qualified_name(),
                    role.attribute().qualified_name(),
                ),
                (role.element(), KnownHtmlAttribute::Id.qualified_name()),
                (
                    QualifiedName::new(Namespace::svg(), None, "x"),
                    QualifiedName::new(Namespace::svg(), None, "charset"),
                ),
            ] {
                let raw = Attribute::new(name, value.clone());
                let error = role
                    .read(units.len() - 1, &element, Some(&raw))
                    .unwrap_err();
                assert_eq!(error.capacity(), units.len() - 1);
                assert_eq!(error.required(), units.len());
                let report = role.read(units.len(), &element, Some(&raw)).unwrap();
                assert_eq!(report.raw().unwrap().code_units(), units);
                assert_eq!(report.raw_attribute(), Some(&raw));
            }
        }
        let long = role.raw(format!("UTF-8{}", " ".repeat(2048)));
        let report = role.read(2053, &role.element(), Some(&long)).unwrap();
        assert_eq!(report.recognition(), HtmlAttributeRecognition::Unrecognized);
        assert_eq!(report.intrinsic(), None);
        let error = role.read(2052, &role.element(), Some(&long)).unwrap_err();
        assert_eq!(error.required(), 2053);
    }
}

#[test]
fn retained_exact_reports_survive_raw_drop_and_later_independent_interpretation() {
    for role in [Role::Form, Role::Meta] {
        let raw = role.raw("uTf-8");
        let report = role.read(5, &role.element(), Some(&raw)).unwrap();
        let historical = report.clone();
        drop(raw);
        let invalid = role.raw("latin1");
        assert_eq!(
            role.read(6, &role.element(), Some(&invalid))
                .unwrap()
                .intrinsic(),
            None
        );
        assert_eq!(report, historical);
        assert_eq!(report.raw().unwrap(), &DocumentString::from("uTf-8"));
        assert_eq!(report.intrinsic().unwrap().value(), report.raw().unwrap());
    }
}
