use surgeist_html::*;

fn raw(value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(KnownHtmlAttribute::Name.qualified_name(), value)
}
fn read(raw: Option<&Attribute>) -> HtmlAttributeReport<HtmlSpaceToken> {
    HtmlValueInterpreter::new(4096)
        .map_name(&KnownHtmlElement::Map.qualified_name(), raw)
        .unwrap()
}
#[test]
fn required_absence_is_missing_not_a_fabricated_empty_token() {
    let r = read(None);
    assert_eq!(r.applicability(), HtmlAttributeApplicability::Applicable);
    assert_eq!(r.recognition(), HtmlAttributeRecognition::Missing);
    assert_eq!(r.author(), HtmlAttributeAuthorState::Missing);
    assert_eq!(
        r.context_author(),
        HtmlAttributeContextAuthorState::NotRequired
    );
    assert_eq!(r.raw(), None);
    assert_eq!(r.intrinsic(), None);
    assert_eq!(r.attribute(), KnownHtmlAttribute::Name);
    assert_eq!(r.inventory_rows(), &[106]);
    assert!(r.source_clause().ends_with("/#attr-map-name"));
}
#[test]
fn empty_and_each_ascii_whitespace_have_independent_local_failures() {
    let a = raw("");
    let r = read(Some(&a));
    assert_eq!(r.recognition(), HtmlAttributeRecognition::Empty);
    assert_eq!(
        r.author(),
        HtmlAttributeAuthorState::Nonconforming(
            HtmlAttributeConformanceError::InvalidMapNameToken(HtmlTokenError::EmptySpaceToken)
        )
    );
    assert_eq!(
        r.context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
    for white in [9, 10, 12, 13, 32] {
        let a = raw(DocumentString::from_utf16(vec![65, white, 66]));
        let r = read(Some(&a));
        assert_eq!(
            r.author(),
            HtmlAttributeAuthorState::Nonconforming(
                HtmlAttributeConformanceError::InvalidMapNameToken(
                    HtmlTokenError::SpaceTokenWhitespace {
                        code_unit_offset: 1
                    }
                )
            )
        );
        assert_eq!(r.recognition(), HtmlAttributeRecognition::Unrecognized);
        assert_eq!(r.raw_attribute(), Some(&a));
        assert_eq!(r.intrinsic(), None);
        assert_eq!(
            r.context_author(),
            HtmlAttributeContextAuthorState::Unchecked
        );
    }
}
#[test]
fn opaque_utf16_and_nonascii_whitespace_are_preserved_and_emitted_exactly() {
    for units in [
        vec![0],
        vec![0xD800],
        vec![0xDC00],
        vec![0xD800, 0xDC00],
        vec![0xA0],
        vec![0x2003],
        vec![65, 58, 66],
    ] {
        let value = DocumentString::from_utf16(units);
        let a = raw(value.clone());
        let r = read(Some(&a));
        assert_eq!(r.author(), HtmlAttributeAuthorState::Conforming);
        assert_eq!(
            r.context_author(),
            HtmlAttributeContextAuthorState::Unchecked
        );
        assert_eq!(r.intrinsic().unwrap().value(), &value);
        let emitted = HtmlSpaceToken::new(value.clone())
            .unwrap()
            .into_map_name_attribute();
        assert_eq!(emitted.name(), &KnownHtmlAttribute::Name.qualified_name());
        assert_eq!(emitted.value(), &value);
        assert_eq!(emitted.name().prefix(), None);
    }
}
#[test]
fn exact_expanded_names_ignore_prefix_but_not_case_or_namespace() {
    let value = raw("map");
    let name = QualifiedName::new(Namespace::html(), Some("h".into()), "map");
    let prefixed = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("p".into()), "name"),
        "map",
    );
    assert_eq!(
        HtmlValueInterpreter::new(3)
            .map_name(&name, Some(&prefixed))
            .unwrap()
            .author(),
        HtmlAttributeAuthorState::Conforming
    );
    for name in [
        KnownHtmlElement::Div.qualified_name(),
        QualifiedName::new(Namespace::html(), None, "MAP"),
    ] {
        let r = HtmlValueInterpreter::new(3)
            .map_name(&name, Some(&value))
            .unwrap();
        assert_eq!(
            r.applicability(),
            HtmlAttributeApplicability::DifferentElement
        );
        assert_eq!(
            r.context_author(),
            HtmlAttributeContextAuthorState::Unapplied
        );
        assert_eq!(r.raw_attribute(), Some(&value));
    }
    for ns in [Namespace::none(), Namespace::svg()] {
        let r = HtmlValueInterpreter::new(3)
            .map_name(&QualifiedName::new(ns, None, "map"), Some(&value))
            .unwrap();
        assert_eq!(
            r.applicability(),
            HtmlAttributeApplicability::ForeignElementNamespace
        );
    }
    for name in [
        QualifiedName::new(Namespace::none(), None, "NAME"),
        QualifiedName::new(Namespace::svg(), None, "name"),
        KnownHtmlAttribute::Id.qualified_name(),
    ] {
        let a = Attribute::new(name, "map");
        let r = read(Some(&a));
        assert_eq!(
            r.applicability(),
            HtmlAttributeApplicability::DifferentAttribute
        );
        assert_eq!(r.recognition(), HtmlAttributeRecognition::Unapplied);
        assert_eq!(r.author(), HtmlAttributeAuthorState::Unapplied);
    }
}
#[test]
fn original_length_cap_precedes_applicability_and_validation() {
    let value = raw("a ");
    assert!(
        HtmlValueInterpreter::new(1)
            .map_name(&KnownHtmlElement::Div.qualified_name(), Some(&value))
            .is_err()
    );
    assert!(
        HtmlValueInterpreter::new(2)
            .map_name(&KnownHtmlElement::Map.qualified_name(), Some(&value))
            .is_ok()
    );
    let wrong = Attribute::new(KnownHtmlAttribute::Id.qualified_name(), "abc");
    assert!(
        HtmlValueInterpreter::new(2)
            .map_name(&KnownHtmlElement::Map.qualified_name(), Some(&wrong))
            .is_err()
    );
}

#[test]
fn first_whitespace_failure_uses_original_utf16_offset_without_scalar_conversion() {
    let a = raw(DocumentString::from_utf16(vec![
        0xD800, 0xDC00, 0xD800, 32, 9,
    ]));
    assert_eq!(
        read(Some(&a)).author(),
        HtmlAttributeAuthorState::Nonconforming(
            HtmlAttributeConformanceError::InvalidMapNameToken(
                HtmlTokenError::SpaceTokenWhitespace {
                    code_unit_offset: 3
                }
            )
        )
    );
}
