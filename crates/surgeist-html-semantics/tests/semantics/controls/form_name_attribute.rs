use surgeist_html::*;
use surgeist_html_semantics::HtmlSemanticQuery;

fn raw(value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(KnownHtmlAttribute::Name.qualified_name(), value)
}
fn read(a: Option<&Attribute>) -> HtmlAttributeReport<HtmlFormName> {
    HtmlValueInterpreter::new(4096)
        .form_name(&KnownHtmlElement::Form.qualified_name(), a)
        .unwrap()
}

#[test]
fn optional_absence_is_missing_observation_and_locally_conforming_without_default() {
    let r = read(None);
    assert_eq!(r.observation(), HtmlAttributeObservation::Missing);
    assert_eq!(r.recognition(), HtmlAttributeRecognition::Missing);
    assert_eq!(r.author(), HtmlAttributeAuthorState::Conforming);
    assert_eq!(
        r.context_author(),
        HtmlAttributeContextAuthorState::NotRequired
    );
    assert_eq!(r.raw_attribute(), None);
    assert_eq!(r.intrinsic(), None);
}

#[test]
fn specified_empty_is_a_local_failure_with_independent_unchecked_collection() {
    let a = raw("");
    let r = read(Some(&a));
    assert_eq!(r.observation(), HtmlAttributeObservation::PresentEmpty);
    assert_eq!(r.recognition(), HtmlAttributeRecognition::Empty);
    assert_eq!(
        r.author(),
        HtmlAttributeAuthorState::Nonconforming(
            HtmlAttributeConformanceError::FormNameMustNotBeEmpty
        )
    );
    assert_eq!(
        r.context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
    assert_eq!(r.raw_attribute(), Some(&a));
    assert_eq!(r.intrinsic(), None);
    assert_eq!(
        HtmlFormName::new(""),
        Err(HtmlAttributeConformanceError::FormNameMustNotBeEmpty)
    );
}

#[test]
fn all_nonempty_exact_units_are_local_names_without_tokenization() {
    // HTML form-name has only the nonempty local restriction, unlike id/map name.
    for units in [
        vec![9],
        vec![10],
        vec![12],
        vec![13],
        vec![32],
        vec![32, 65, 32],
        vec![0],
        vec![0xD800],
        vec![0xDC00],
        vec![0xD83D, 0xDE00],
        vec![0xD800, 0, 32, 0xDC00],
        vec![0xA0],
    ] {
        let value = DocumentString::from_utf16(units.clone());
        let a = raw(value.clone());
        let r = read(Some(&a));
        assert_eq!(r.author(), HtmlAttributeAuthorState::Conforming);
        assert_eq!(r.recognition(), HtmlAttributeRecognition::NonemptyText);
        assert_eq!(
            r.context_author(),
            HtmlAttributeContextAuthorState::Unchecked
        );
        assert_eq!(r.intrinsic().unwrap().value(), &value);
        assert_eq!(r.raw().unwrap().code_units(), units);
        let emitted = HtmlFormName::new(value).unwrap().into_attribute();
        assert_eq!(emitted.name(), &KnownHtmlAttribute::Name.qualified_name());
        assert_eq!(emitted.name().prefix(), None);
        assert_eq!(emitted.value().code_units(), units);
    }
}

#[test]
fn native_names_do_not_fold_trim_or_normalize_spelling() {
    let a = HtmlFormName::new("x").unwrap();
    assert_eq!(a, HtmlFormName::new(String::from("x")).unwrap());
    assert_eq!(a.clone().value(), a.value());
    for value in ["X", " x", "x ", "x\0"] {
        assert_ne!(a, HtmlFormName::new(value).unwrap());
    }
    assert_ne!(
        HtmlFormName::new("é").unwrap(),
        HtmlFormName::new("e\u{301}").unwrap()
    );
}

#[test]
fn expanded_name_identity_ignores_prefix_but_rejects_uppercase_and_namespace() {
    let e = QualifiedName::new(Namespace::html(), Some("h".into()), "form");
    let a = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("p".into()), "name"),
        "x",
    );
    let r = HtmlValueInterpreter::new(1)
        .form_name(&e, Some(&a))
        .unwrap();
    assert_eq!(r.author(), HtmlAttributeAuthorState::Conforming);
    assert_eq!(r.raw_attribute(), Some(&a));
    for name in [
        QualifiedName::new(Namespace::none(), None, "NAME"),
        QualifiedName::new(Namespace::svg(), None, "name"),
    ] {
        let a = Attribute::new(name, "x");
        let r = HtmlValueInterpreter::new(1)
            .form_name(&e, Some(&a))
            .unwrap();
        assert_eq!(
            r.applicability(),
            HtmlAttributeApplicability::DifferentAttribute
        );
        assert_eq!(r.recognition(), HtmlAttributeRecognition::Unapplied);
        assert_eq!(r.author(), HtmlAttributeAuthorState::Unapplied);
        assert_eq!(r.raw_attribute(), Some(&a));
    }
}

#[test]
fn wrong_element_roles_are_unapplied_with_raw_preserved() {
    let a = raw("x");
    for local in ["FORM", "input", "map", "x-form"] {
        let r = HtmlValueInterpreter::new(1)
            .form_name(
                &QualifiedName::new(Namespace::html(), None, local),
                Some(&a),
            )
            .unwrap();
        assert_eq!(
            r.applicability(),
            HtmlAttributeApplicability::DifferentElement
        );
        assert_eq!(r.author(), HtmlAttributeAuthorState::Unapplied);
        assert_eq!(
            r.context_author(),
            HtmlAttributeContextAuthorState::Unapplied
        );
        assert_eq!(r.raw_attribute(), Some(&a));
    }
    for ns in [Namespace::none(), Namespace::svg()] {
        let r = HtmlValueInterpreter::new(1)
            .form_name(&QualifiedName::new(ns, None, "form"), Some(&a))
            .unwrap();
        assert_eq!(
            r.applicability(),
            HtmlAttributeApplicability::ForeignElementNamespace
        );
    }
}

#[test]
fn whole_original_utf16_admission_precedes_even_wrong_role_checks() {
    let a = raw(DocumentString::from_utf16(vec![0xD800, 0, 0xDC00]));
    assert!(
        HtmlValueInterpreter::new(3)
            .form_name(&KnownHtmlElement::Form.qualified_name(), Some(&a))
            .is_ok()
    );
    for e in [
        KnownHtmlElement::Form.qualified_name(),
        KnownHtmlElement::Div.qualified_name(),
        QualifiedName::new(Namespace::svg(), None, "form"),
    ] {
        assert!(
            HtmlValueInterpreter::new(2)
                .form_name(&e, Some(&a))
                .is_err()
        );
    }
}

#[test]
fn report_binds_defining_clause_and_original_inventory_role_without_context_certificate() {
    let a = raw("same");
    let r = read(Some(&a));
    assert_eq!(r.attribute(), KnownHtmlAttribute::Name);
    assert_eq!(r.inventory_rows(), &[104]);
    assert!(r.source_clause().ends_with("/#attr-form-name"));
    assert_eq!(
        r.source_clause(),
        HtmlSemanticQuery::FormNameAuthor.source_clause()
    );
    assert_eq!(
        r.context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
    assert_eq!(read(Some(&a)), r);
}
