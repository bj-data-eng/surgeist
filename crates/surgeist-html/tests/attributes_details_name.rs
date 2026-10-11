use surgeist_html::{
    Attribute, DocumentString, HtmlAttributeApplicability, HtmlAttributeAuthorState,
    HtmlAttributeConformanceError, HtmlAttributeContextAuthorState, HtmlAttributeObservation,
    HtmlAttributeRecognition, HtmlAttributeReport, HtmlBooleanAttribute, HtmlDetailsName,
    HtmlValueInterpreter, KnownHtmlAttribute, KnownHtmlElement, Namespace, QualifiedName,
};

fn raw(value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(KnownHtmlAttribute::Name.qualified_name(), value)
}

fn read(raw: Option<&Attribute>) -> HtmlAttributeReport<HtmlDetailsName> {
    HtmlValueInterpreter::new(4096)
        .details_name(&KnownHtmlElement::Details.qualified_name(), raw)
        .unwrap()
}

fn conforming(report: &HtmlAttributeReport<HtmlDetailsName>, value: &DocumentString) {
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
    assert_eq!(report.raw(), Some(value));
    assert_eq!(report.intrinsic().unwrap().value(), value);
    assert_eq!(
        report.context_author(),
        HtmlAttributeContextAuthorState::NotRequired
    );
}

#[test]
fn optional_absence_and_specified_empty_have_distinct_author_outcomes() {
    // #attr-details-name permits absence, but forbids a specified empty value.
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
    assert_eq!(missing.raw(), None);
    assert_eq!(missing.intrinsic(), None);

    let empty = raw("");
    let report = read(Some(&empty));
    assert_eq!(report.observation(), HtmlAttributeObservation::PresentEmpty);
    assert_eq!(report.recognition(), HtmlAttributeRecognition::Empty);
    assert_eq!(
        report.author(),
        HtmlAttributeAuthorState::Nonconforming(
            HtmlAttributeConformanceError::DetailsNameMustNotBeEmpty,
        )
    );
    assert_eq!(
        report.context_author(),
        HtmlAttributeContextAuthorState::NotRequired
    );
    assert_eq!(report.raw_attribute(), Some(&empty));
    assert_eq!(report.raw(), Some(&DocumentString::default()));
    assert_eq!(report.intrinsic(), None);
    assert_eq!(
        HtmlDetailsName::new(""),
        Err(HtmlAttributeConformanceError::DetailsNameMustNotBeEmpty)
    );
    assert_eq!(
        HtmlDetailsName::new(DocumentString::default()),
        Err(HtmlAttributeConformanceError::DetailsNameMustNotBeEmpty)
    );
}

#[test]
fn whitespace_punctuation_and_keyword_like_names_are_exact_nonempty_text() {
    // The clause excludes only empty; it does not select a token grammar.
    for text in [
        "x",
        "X",
        " ",
        "\t",
        "\n",
        "\r",
        "\u{c}",
        " \t\r\n\u{c} ",
        " x ",
        "two words",
        ",",
        "open",
        "false",
        "name",
        "NAME",
        "\u{a0}",
        "\u{2003}",
        "\0",
        " a,b ",
    ] {
        let attribute = raw(text);
        conforming(&read(Some(&attribute)), &DocumentString::from(text));
        let checked = HtmlDetailsName::new(text).unwrap();
        assert_eq!(checked.value(), &DocumentString::from(text));
        let emitted = checked.into_attribute();
        assert_eq!(emitted.name(), &KnownHtmlAttribute::Name.qualified_name());
        assert_eq!(emitted.value(), &DocumentString::from(text));
        conforming(&read(Some(&emitted)), &DocumentString::from(text));
    }
}

#[test]
fn native_equality_does_not_fold_case_trim_spaces_or_normalize_unicode() {
    let x = HtmlDetailsName::new("x").unwrap();
    assert_eq!(x, HtmlDetailsName::new(String::from("x")).unwrap());
    assert_eq!(x.clone().value(), x.value());
    for text in ["X", " x", "x ", "x\0"] {
        assert_ne!(x, HtmlDetailsName::new(text).unwrap());
    }
    assert_ne!(
        HtmlDetailsName::new("é").unwrap(),
        HtmlDetailsName::new("e\u{301}").unwrap()
    );
    assert_ne!(
        HtmlDetailsName::new(" ").unwrap(),
        HtmlDetailsName::new("\u{a0}").unwrap()
    );
}

#[test]
fn isolated_and_supplementary_utf16_names_preserve_every_unit() {
    for units in [
        vec![0xD800],
        vec![0xDC00],
        vec![0xD83D, 0xDE00],
        vec![0xD800, 0x20, 0xDC00, 0, 0xD83D, 0xDE00],
    ] {
        let value = DocumentString::from_utf16(units.clone());
        let attribute = raw(value.clone());
        let report = read(Some(&attribute));
        conforming(&report, &value);
        assert_eq!(report.raw().unwrap().code_units(), units);
        assert_eq!(report.intrinsic().unwrap().value().code_units(), units);
        let emitted = HtmlDetailsName::new(value).unwrap().into_attribute();
        assert_eq!(emitted.value().code_units(), units);
        assert_eq!(emitted.name().namespace(), &Namespace::none());
        assert_eq!(emitted.name().prefix(), None);
        assert_eq!(emitted.name().local_name(), &DocumentString::from("name"));
    }
}

#[test]
fn defining_metadata_and_raw_prefixes_do_not_replace_expanded_identity() {
    let element = QualifiedName::new(Namespace::html(), Some("html-prefix".into()), "details");
    let attribute = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("raw-prefix".into()), "name"),
        " group ",
    );
    let report = HtmlValueInterpreter::new(64)
        .details_name(&element, Some(&attribute))
        .unwrap();
    conforming(&report, &DocumentString::from(" group "));
    assert_eq!(report.element(), &element);
    assert_eq!(report.raw_attribute(), Some(&attribute));
    assert_eq!(report.attribute(), KnownHtmlAttribute::Name);
    assert_eq!(report.inventory_rows(), &[103]);
    assert_eq!(
        report.source_clause(),
        "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-details-name"
    );
    assert_eq!(report.vocabulary_source_clause(), None);
    assert_eq!(report.script_context(), None);
    assert_eq!(report.hyperlink_context(), None);
}

#[test]
fn wrong_element_namespace_case_and_attribute_keep_exact_unapplied_evidence() {
    let name = raw("");
    let details = KnownHtmlElement::Details.qualified_name();
    let wrong_attribute = Attribute::new(KnownHtmlAttribute::Open.qualified_name(), "");
    let foreign_attribute = Attribute::new(
        QualifiedName::new(Namespace::svg(), Some("s".into()), "name"),
        " x ",
    );
    let uppercase_attribute = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("p".into()), "NAME"),
        "",
    );
    for (element, attribute, applicability) in [
        (
            KnownHtmlElement::Div.qualified_name(),
            &name,
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            KnownHtmlElement::Dialog.qualified_name(),
            &name,
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            KnownHtmlElement::Form.qualified_name(),
            &name,
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            QualifiedName::new(Namespace::html(), None, "DETAILS"),
            &name,
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            QualifiedName::new(Namespace::html(), None, "x-details"),
            &name,
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            QualifiedName::new(Namespace::svg(), Some("s".into()), "details"),
            &name,
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
        (
            QualifiedName::new(Namespace::none(), None, "details"),
            &name,
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
        (
            details.clone(),
            &wrong_attribute,
            HtmlAttributeApplicability::DifferentAttribute,
        ),
        (
            details.clone(),
            &foreign_attribute,
            HtmlAttributeApplicability::DifferentAttribute,
        ),
        (
            details,
            &uppercase_attribute,
            HtmlAttributeApplicability::DifferentAttribute,
        ),
    ] {
        let report = HtmlValueInterpreter::new(64)
            .details_name(&element, Some(attribute))
            .unwrap();
        assert_eq!(report.applicability(), applicability);
        assert_eq!(report.element(), &element);
        assert_eq!(report.raw_attribute(), Some(attribute));
        assert_eq!(report.raw(), Some(attribute.value()));
        assert_eq!(
            report.observation(),
            if attribute.value().is_empty() {
                HtmlAttributeObservation::PresentEmpty
            } else {
                HtmlAttributeObservation::PresentNonempty
            }
        );
        assert_eq!(report.recognition(), HtmlAttributeRecognition::Unapplied);
        assert_eq!(report.author(), HtmlAttributeAuthorState::Unapplied);
        assert_eq!(
            report.context_author(),
            HtmlAttributeContextAuthorState::Unapplied
        );
        assert_eq!(report.intrinsic(), None);
        assert_eq!(report.inventory_rows(), &[103]);
    }
    let absent = HtmlValueInterpreter::new(0)
        .details_name(&KnownHtmlElement::Div.qualified_name(), None)
        .unwrap();
    assert_eq!(absent.observation(), HtmlAttributeObservation::Missing);
    assert_eq!(absent.author(), HtmlAttributeAuthorState::Unapplied);
    assert_eq!(absent.recognition(), HtmlAttributeRecognition::Unapplied);
    assert_eq!(absent.intrinsic(), None);
}

#[test]
fn original_utf16_cap_precedes_applicability_and_counts_units_without_loss() {
    let details = KnownHtmlElement::Details.qualified_name();
    for units in [
        vec![0xD800],
        vec![0xDC00],
        vec![0xD83D, 0xDE00],
        vec![0xD800, 0, 0xDC00],
    ] {
        let value = DocumentString::from_utf16(units.clone());
        for (element, name) in [
            (details.clone(), KnownHtmlAttribute::Name.qualified_name()),
            (
                KnownHtmlElement::Div.qualified_name(),
                KnownHtmlAttribute::Name.qualified_name(),
            ),
            (details.clone(), KnownHtmlAttribute::Open.qualified_name()),
            (
                QualifiedName::new(Namespace::svg(), None, "details"),
                QualifiedName::new(Namespace::svg(), Some("s".into()), "name"),
            ),
        ] {
            let attribute = Attribute::new(name, value.clone());
            let error = HtmlValueInterpreter::new(units.len() - 1)
                .details_name(&element, Some(&attribute))
                .unwrap_err();
            assert_eq!(error.capacity(), units.len() - 1);
            assert_eq!(error.required(), units.len());
            let report = HtmlValueInterpreter::new(units.len())
                .details_name(&element, Some(&attribute))
                .unwrap();
            assert_eq!(report.raw().unwrap().code_units(), units);
            assert_eq!(report.raw_attribute(), Some(&attribute));
        }
    }
    let interpreter = HtmlValueInterpreter::new(0);
    assert_eq!(
        interpreter.details_name(&details, None).unwrap().author(),
        HtmlAttributeAuthorState::Missing
    );
    let empty = raw("");
    assert_eq!(
        interpreter
            .details_name(&details, Some(&empty))
            .unwrap()
            .author(),
        HtmlAttributeAuthorState::Nonconforming(
            HtmlAttributeConformanceError::DetailsNameMustNotBeEmpty
        )
    );
    let nonempty = raw(" ");
    let error = interpreter
        .details_name(&details, Some(&nonempty))
        .unwrap_err();
    assert_eq!(error.capacity(), 0);
    assert_eq!(error.required(), 1);
}

#[test]
fn names_at_the_admitted_limit_remain_whole_not_tokens_or_prefixes() {
    let value = DocumentString::from(" x,\t".repeat(1024));
    let attribute = raw(value.clone());
    let report = HtmlValueInterpreter::new(4096)
        .details_name(
            &KnownHtmlElement::Details.qualified_name(),
            Some(&attribute),
        )
        .unwrap();
    conforming(&report, &value);
    let error = HtmlValueInterpreter::new(4095)
        .details_name(
            &KnownHtmlElement::Details.qualified_name(),
            Some(&attribute),
        )
        .unwrap_err();
    assert_eq!(error.capacity(), 4095);
    assert_eq!(error.required(), 4096);
    assert_eq!(
        HtmlDetailsName::new(value.clone())
            .unwrap()
            .into_attribute()
            .value(),
        &value
    );
}

#[test]
fn retained_reports_and_native_emission_survive_later_unrelated_construction() {
    let first = raw(DocumentString::from_utf16(vec![0x20, 0xD800, 0x78, 0x20]));
    let report = read(Some(&first));
    let clone = report.clone();
    drop(first);
    let later = raw("OTHER");
    let later_report = read(Some(&later));
    conforming(&later_report, &DocumentString::from("OTHER"));
    assert_eq!(report, clone);
    assert_eq!(
        report.raw().unwrap().code_units(),
        &[0x20, 0xD800, 0x78, 0x20]
    );
    let emitted = report.intrinsic().unwrap().clone().into_attribute();
    assert_eq!(emitted.value().code_units(), &[0x20, 0xD800, 0x78, 0x20]);
    conforming(&read(Some(&emitted)), emitted.value());
}

#[test]
fn checked_name_does_not_replace_details_open_boolean_spelling_or_presence() {
    let interpreter = HtmlValueInterpreter::new(64);
    let details = KnownHtmlElement::Details.qualified_name();
    let name = raw("false");
    conforming(
        &interpreter.details_name(&details, Some(&name)).unwrap(),
        &DocumentString::from("false"),
    );
    let open = Attribute::new(KnownHtmlAttribute::Open.qualified_name(), "false");
    let report = interpreter
        .boolean_attribute(HtmlBooleanAttribute::DetailsOpen, &details, Some(&open))
        .unwrap();
    assert_eq!(report.intrinsic(), Some(&true));
    assert_eq!(
        report.author(),
        HtmlAttributeAuthorState::Nonconforming(HtmlAttributeConformanceError::BooleanSpelling)
    );
    assert_eq!(report.raw_attribute(), Some(&open));
    let absent = interpreter
        .boolean_attribute(HtmlBooleanAttribute::DetailsOpen, &details, None)
        .unwrap();
    assert_eq!(absent.intrinsic(), Some(&false));
}
