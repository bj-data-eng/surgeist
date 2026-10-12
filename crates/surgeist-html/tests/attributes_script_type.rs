use surgeist_html::*;

fn script() -> QualifiedName {
    KnownHtmlElement::Script.qualified_name()
}
fn raw(value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(KnownHtmlAttribute::Type.qualified_name(), value)
}
fn attributes(kind: Option<&str>, language: Option<&str>) -> OrderedAttributes {
    OrderedAttributes::new(
        kind.map(raw).into_iter().chain(
            language.map(|v| {
                Attribute::new(QualifiedName::new(Namespace::none(), None, "language"), v)
            }),
        ),
    )
    .unwrap()
}
fn interpreter() -> HtmlValueInterpreter {
    HtmlValueInterpreter::new(1000)
}

#[test]
fn native_keywords_emit_only_the_three_canonical_authored_keywords() {
    for (keyword, text, kind) in [
        (
            HtmlScriptTypeKeyword::Module,
            "module",
            HtmlScriptKind::Module,
        ),
        (
            HtmlScriptTypeKeyword::ImportMap,
            "importmap",
            HtmlScriptKind::ImportMap,
        ),
        (
            HtmlScriptTypeKeyword::SpeculationRules,
            "speculationrules",
            HtmlScriptKind::SpeculationRules,
        ),
    ] {
        let attribute = keyword.into_attribute();
        assert_eq!(keyword.keyword(), text);
        assert_eq!(attribute.name(), &KnownHtmlAttribute::Type.qualified_name());
        assert_eq!(attribute.value(), &DocumentString::from(text));
        let report = interpreter()
            .script_type(&script(), Some(&attribute))
            .unwrap();
        assert_eq!(report.meaning(), Some(HtmlScriptTypeMeaning::Script(kind)));
        assert_eq!(report.author(), HtmlScriptTypeAuthorState::HtmlDefined);
        assert_eq!(report.author_advice(), HtmlScriptTypeAuthorAdvice::None);
    }
}
#[test]
fn missing_and_empty_are_distinct_nominal_classic_observations_and_advice() {
    let i = interpreter();
    let missing = i.script_type(&script(), None).unwrap();
    let empty_raw = raw("");
    let empty = i.script_type(&script(), Some(&empty_raw)).unwrap();
    assert_eq!(missing.observation(), HtmlAttributeObservation::Missing);
    assert_eq!(empty.observation(), HtmlAttributeObservation::PresentEmpty);
    assert_eq!(missing.raw_attribute(), None);
    assert_eq!(empty.raw_attribute(), Some(&empty_raw));
    for r in [&missing, &empty] {
        assert_eq!(
            r.meaning(),
            Some(HtmlScriptTypeMeaning::Script(HtmlScriptKind::Classic))
        );
        assert_eq!(r.author(), HtmlScriptTypeAuthorState::HtmlDefined);
        assert_eq!(r.inventory_row_indices(), &[171]);
        assert_eq!(r.attribute(), KnownHtmlAttribute::Type);
        assert!(r.source_clause().ends_with("/#attr-script-type"));
    }
    assert_eq!(missing.author_advice(), HtmlScriptTypeAuthorAdvice::None);
    assert_eq!(
        empty.author_advice(),
        HtmlScriptTypeAuthorAdvice::OmitRedundantType
    );
    assert_eq!(
        missing.context_author(),
        HtmlAttributeContextAuthorState::NotRequired
    );
    assert_eq!(
        empty.context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
}
#[test]
fn all_sixteen_independently_named_mime_essences_are_exact_classic_matches() {
    // Independent expectations from MIME39aa535 §4.6, not the provider table.
    for value in [
        "application/ecmascript",
        "application/javascript",
        "application/x-ecmascript",
        "application/x-javascript",
        "text/ecmascript",
        "text/javascript",
        "text/javascript1.0",
        "text/javascript1.1",
        "text/javascript1.2",
        "text/javascript1.3",
        "text/javascript1.4",
        "text/javascript1.5",
        "text/jscript",
        "text/livescript",
        "text/x-ecmascript",
        "text/x-javascript",
    ] {
        for value in [value.to_owned(), value.to_ascii_uppercase()] {
            let attribute = raw(value);
            let r = interpreter()
                .script_type(&script(), Some(&attribute))
                .unwrap();
            assert_eq!(
                r.meaning(),
                Some(HtmlScriptTypeMeaning::Script(HtmlScriptKind::Classic))
            );
            assert_eq!(r.author(), HtmlScriptTypeAuthorState::HtmlDefined);
            assert_eq!(
                r.author_advice(),
                HtmlScriptTypeAuthorAdvice::OmitRedundantType
            );
            assert_eq!(r.raw_attribute(), Some(&attribute));
        }
    }
}
#[test]
fn keyword_casing_is_ascii_only_and_whitespace_does_not_trim_keywords() {
    for (value, kind) in [
        ("MoDuLe", HtmlScriptKind::Module),
        ("IMPORTMAP", HtmlScriptKind::ImportMap),
        ("sPeCuLaTiOnRuLeS", HtmlScriptKind::SpeculationRules),
    ] {
        let r = interpreter()
            .script_type(&script(), Some(&raw(value)))
            .unwrap();
        assert_eq!(r.meaning(), Some(HtmlScriptTypeMeaning::Script(kind)));
    }
    for value in [
        " module ",
        "\timportmap\n",
        " speculationrules ",
        "moduℓe",
        "classic",
        "data-block",
    ] {
        let a = attributes(Some(value), None);
        let r = interpreter()
            .script_type_with_attributes(&script(), &a)
            .unwrap();
        assert_eq!(
            r.content().meaning(),
            Some(HtmlScriptTypeMeaning::DataBlock)
        );
        assert_eq!(
            r.content().author(),
            HtmlScriptTypeAuthorState::ExternalMimeUnchecked
        );
        assert_eq!(r.classification(), HtmlScriptTypeClassification::NoScript);
    }
}
#[test]
fn data_block_mime_grammar_is_unchecked_independently_of_no_script_classification() {
    for value in [
        "application/json",
        "text/html",
        "text/html;",
        "text/javascript;charset=utf-8",
        "text/javascript1.6",
        "unknown",
        "   ",
    ] {
        let a = attributes(Some(value), None);
        let r = interpreter()
            .script_type_with_attributes(&script(), &a)
            .unwrap();
        assert_eq!(
            r.content().meaning(),
            Some(HtmlScriptTypeMeaning::DataBlock)
        );
        assert_eq!(
            r.content().author(),
            HtmlScriptTypeAuthorState::ExternalMimeUnchecked
        );
        assert_eq!(
            r.content().context_author(),
            HtmlAttributeContextAuthorState::Unchecked
        );
        assert_eq!(r.classification(), HtmlScriptTypeClassification::NoScript);
        assert_eq!(
            r.content().raw_attribute().unwrap().value(),
            &DocumentString::from(value)
        );
    }
}
#[test]
fn explicit_js_whitespace_exception_changes_only_preparation_classification() {
    let text = " \tTEXT/JAVASCRIPT\n\r\x0c ";
    let a = attributes(Some(text), None);
    let r = interpreter()
        .script_type_with_attributes(&script(), &a)
        .unwrap();
    assert_eq!(
        r.content().meaning(),
        Some(HtmlScriptTypeMeaning::DataBlock)
    );
    assert_eq!(
        r.content().author(),
        HtmlScriptTypeAuthorState::ExternalMimeUnchecked
    );
    assert_eq!(
        r.content().author_advice(),
        HtmlScriptTypeAuthorAdvice::None
    );
    assert_eq!(
        r.classification(),
        HtmlScriptTypeClassification::Script(HtmlScriptKind::Classic)
    );
    assert_eq!(
        r.content().raw_attribute().unwrap().value(),
        &DocumentString::from(text)
    );
    for text in [
        "\u{a0}text/javascript\u{a0}",
        "\u{2003}text/javascript\u{2003}",
    ] {
        assert_eq!(
            interpreter()
                .script_type_with_attributes(&script(), &attributes(Some(text), None))
                .unwrap()
                .classification(),
            HtmlScriptTypeClassification::NoScript
        );
    }
}
#[test]
fn actual_complete_language_slots_and_type_override_are_not_guessed_defaults() {
    for (kind, language, expected) in [
        (None, None, Some(HtmlScriptKind::Classic)),
        (None, Some(""), Some(HtmlScriptKind::Classic)),
        (None, Some("JavaScript"), Some(HtmlScriptKind::Classic)),
        (None, Some("jscript"), Some(HtmlScriptKind::Classic)),
        (None, Some(" javascript "), None),
        (None, Some("module"), None),
        (Some(""), Some("unknown"), Some(HtmlScriptKind::Classic)),
        (
            Some("module"),
            Some("unknown"),
            Some(HtmlScriptKind::Module),
        ),
        (Some("unknown"), Some("javascript"), None),
    ] {
        let a = attributes(kind, language);
        let r = interpreter()
            .script_type_with_attributes(&script(), &a)
            .unwrap();
        assert_eq!(
            r.classification(),
            expected.map_or(
                HtmlScriptTypeClassification::NoScript,
                HtmlScriptTypeClassification::Script
            ),
            "{kind:?}/{language:?}"
        );
        assert_eq!(
            r.language_attribute().map(Attribute::value),
            language.map(DocumentString::from).as_ref()
        );
        if kind.is_none() {
            assert_eq!(
                r.content().meaning(),
                Some(HtmlScriptTypeMeaning::Script(HtmlScriptKind::Classic))
            );
        }
        assert!(r.source_clause().ends_with("/#script-processing-prepare"));
    }
}
#[test]
fn exact_utf16_remains_symbolic_and_no_unicode_replacement_or_scalar_folding_occurs() {
    for units in [
        vec![0],
        vec![0xd800],
        vec![0xdc00],
        vec![0xd83d, 0xde00],
        "text/javaſcript".encode_utf16().collect(),
    ] {
        let value = DocumentString::from_utf16(units);
        let a = OrderedAttributes::new([raw(value.clone())]).unwrap();
        let r = interpreter()
            .script_type_with_attributes(&script(), &a)
            .unwrap();
        assert_eq!(r.content().raw_attribute().unwrap().value(), &value);
        assert_eq!(
            r.content().author(),
            HtmlScriptTypeAuthorState::ExternalMimeUnchecked
        );
        assert_eq!(r.classification(), HtmlScriptTypeClassification::NoScript);
    }
}
#[test]
fn exact_expanded_roles_and_prefix_metadata_do_not_normalize_raw_names() {
    let i = interpreter();
    for (ns, local, expected) in [
        (
            Namespace::html(),
            "div",
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            Namespace::html(),
            "SCRIPT",
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            Namespace::svg(),
            "script",
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
    ] {
        let r = i
            .script_type(&QualifiedName::new(ns, None, local), Some(&raw("module")))
            .unwrap();
        assert_eq!(r.applicability(), expected);
        assert_eq!(r.meaning(), None);
        assert_eq!(r.author(), HtmlScriptTypeAuthorState::Unapplied);
    }
    for (ns, local) in [
        (Namespace::none(), "TYPE"),
        (Namespace::svg(), "type"),
        (Namespace::none(), "id"),
    ] {
        let attribute = Attribute::new(QualifiedName::new(ns, None, local), "module");
        let r = i.script_type(&script(), Some(&attribute)).unwrap();
        assert_eq!(
            r.applicability(),
            HtmlAttributeApplicability::DifferentAttribute
        );
        assert_eq!(r.raw_attribute(), Some(&attribute));
    }
    let attribute = Attribute::new(
        QualifiedName::new(
            Namespace::none(),
            Some(DocumentString::from("kept")),
            "type",
        ),
        "module",
    );
    let subject = QualifiedName::new(
        Namespace::html(),
        Some(DocumentString::from("html")),
        "script",
    );
    assert_eq!(
        i.script_type(&subject, Some(&attribute)).unwrap().meaning(),
        Some(HtmlScriptTypeMeaning::Script(HtmlScriptKind::Module))
    );
}
#[test]
fn full_attribute_selection_ignores_wrong_expanded_type_and_language_names() {
    let a = OrderedAttributes::new([
        Attribute::new(
            QualifiedName::new(Namespace::none(), None, "TYPE"),
            "unknown",
        ),
        Attribute::new(
            QualifiedName::new(Namespace::svg(), None, "language"),
            "unknown",
        ),
        Attribute::new(
            QualifiedName::new(Namespace::none(), None, "LANGUAGE"),
            "unknown",
        ),
    ])
    .unwrap();
    let r = interpreter()
        .script_type_with_attributes(&script(), &a)
        .unwrap();
    assert_eq!(r.content().raw_attribute(), None);
    assert_eq!(r.language_attribute(), None);
    assert_eq!(
        r.classification(),
        HtmlScriptTypeClassification::Script(HtmlScriptKind::Classic)
    );
}
#[test]
fn both_original_selected_caps_precede_role_override_or_processing() {
    let i = HtmlValueInterpreter::new(6);
    let wrong = KnownHtmlElement::Div.qualified_name();
    assert!(i.script_type(&wrong, Some(&raw("oversized"))).is_err());
    assert!(
        i.script_type_with_attributes(&wrong, &attributes(Some("oversized"), None))
            .is_err()
    );
    assert!(
        i.script_type_with_attributes(&wrong, &attributes(Some("module"), Some("oversized")))
            .is_err()
    );
    assert!(
        i.script_type_with_attributes(&script(), &attributes(Some("module"), Some("oversized")))
            .is_err()
    );
    assert!(i.script_type(&script(), Some(&raw("module"))).is_ok());
    assert!(
        HtmlValueInterpreter::new(5)
            .script_type(&script(), Some(&raw("module")))
            .is_err()
    );
    let value = DocumentString::from_utf16(vec![0xd83d, 0xde00]);
    assert!(
        HtmlValueInterpreter::new(2)
            .script_type(&script(), Some(&raw(value.clone())))
            .is_ok()
    );
    assert!(
        HtmlValueInterpreter::new(1)
            .script_type(&script(), Some(&raw(value)))
            .is_err()
    );
}
#[test]
fn later_independent_queries_do_not_rewrite_old_report_or_input() {
    let a = attributes(Some("module"), Some("javascript"));
    let snapshot = a.clone();
    let old = interpreter()
        .script_type_with_attributes(&script(), &a)
        .unwrap();
    let later = interpreter()
        .script_type_with_attributes(&script(), &attributes(Some("unknown"), None))
        .unwrap();
    assert_eq!(a, snapshot);
    assert_eq!(
        old.classification(),
        HtmlScriptTypeClassification::Script(HtmlScriptKind::Module)
    );
    assert_eq!(
        later.classification(),
        HtmlScriptTypeClassification::NoScript
    );
}
