use surgeist_html::*;

fn element() -> QualifiedName {
    KnownHtmlElement::Meta.qualified_name()
}
fn raw(value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(KnownHtmlAttribute::HttpEquiv.qualified_name(), value)
}
#[test]
fn all_seven_intrinsic_states_keep_recognition_separate_from_author_conformance() {
    for (spelling, state, conforming) in [
        (
            "content-language",
            HtmlMetaPragmaState::ContentLanguage,
            false,
        ),
        (
            "content-type",
            HtmlMetaPragmaState::EncodingDeclaration,
            true,
        ),
        ("default-style", HtmlMetaPragmaState::DefaultStyle, true),
        ("refresh", HtmlMetaPragmaState::Refresh, true),
        ("set-cookie", HtmlMetaPragmaState::SetCookie, false),
        ("x-ua-compatible", HtmlMetaPragmaState::XUaCompatible, true),
        (
            "content-security-policy",
            HtmlMetaPragmaState::ContentSecurityPolicy,
            true,
        ),
    ] {
        for value in [spelling.to_owned(), spelling.to_ascii_uppercase()] {
            let a = raw(value.clone());
            let r = HtmlValueInterpreter::new(30)
                .meta_http_equiv(&element(), Some(&a))
                .unwrap();
            assert_eq!(r.intrinsic(), Some(&state));
            assert_eq!(r.raw_attribute(), Some(&a));
            assert_eq!(
                r.recognition(),
                HtmlAttributeRecognition::Keyword {
                    matched: spelling,
                    canonical: spelling
                }
            );
            assert_eq!(
                r.author(),
                if conforming {
                    HtmlAttributeAuthorState::Conforming
                } else {
                    HtmlAttributeAuthorState::Nonconforming(
                        HtmlAttributeConformanceError::NonconformingKeyword,
                    )
                }
            );
            assert_eq!(
                r.context_author(),
                HtmlAttributeContextAuthorState::Unchecked
            );
            assert_eq!(r.raw().unwrap(), &DocumentString::from(value));
        }
    }
}
#[test]
fn missing_empty_and_unrecognized_have_no_invented_default_state() {
    let r = HtmlValueInterpreter::new(0)
        .meta_http_equiv(&element(), None::<&surgeist_html::Attribute>)
        .unwrap();
    assert_eq!(r.intrinsic(), None);
    assert_eq!(r.recognition(), HtmlAttributeRecognition::Missing);
    assert_eq!(r.author(), HtmlAttributeAuthorState::Missing);
    for value in [
        "",
        " refresh",
        "refresh ",
        "refresh\0",
        "refreſh",
        "refresh\u{a0}",
        "other",
    ] {
        let a = raw(value);
        let r = HtmlValueInterpreter::new(30)
            .meta_http_equiv(&element(), Some(&a))
            .unwrap();
        assert_eq!(r.intrinsic(), None);
        assert_eq!(
            r.recognition(),
            if value.is_empty() {
                HtmlAttributeRecognition::Empty
            } else {
                HtmlAttributeRecognition::Unrecognized
            }
        );
        assert_eq!(
            r.author(),
            HtmlAttributeAuthorState::Nonconforming(
                HtmlAttributeConformanceError::UnrecognizedKeyword
            )
        );
        assert_eq!(
            r.context_author(),
            HtmlAttributeContextAuthorState::Unchecked
        );
    }
}
#[test]
fn five_native_author_keywords_emit_canonical_attributes() {
    for (native, keyword) in [
        (HtmlMetaPragma::ContentType, "content-type"),
        (HtmlMetaPragma::DefaultStyle, "default-style"),
        (HtmlMetaPragma::Refresh, "refresh"),
        (HtmlMetaPragma::XUaCompatible, "x-ua-compatible"),
        (
            HtmlMetaPragma::ContentSecurityPolicy,
            "content-security-policy",
        ),
    ] {
        assert_eq!(native.canonical_keyword(), keyword);
        let a = native.into_attribute();
        assert_eq!(a.name(), &KnownHtmlAttribute::HttpEquiv.qualified_name());
        assert_eq!(a.value(), &DocumentString::from(keyword));
        assert_eq!(
            HtmlValueInterpreter::new(30)
                .meta_http_equiv(&element(), Some(&a))
                .unwrap()
                .author(),
            HtmlAttributeAuthorState::Conforming
        );
    }
}
#[test]
fn expanded_names_roles_and_prefixes_are_exact() {
    let a = raw("refresh");
    for (name, applicable) in [
        (element(), true),
        (
            QualifiedName::new(Namespace::html(), Some("p".into()), "meta"),
            true,
        ),
        (KnownHtmlElement::Div.qualified_name(), false),
        (QualifiedName::new(Namespace::svg(), None, "meta"), false),
        (QualifiedName::new(Namespace::html(), None, "META"), false),
    ] {
        let r = HtmlValueInterpreter::new(7)
            .meta_http_equiv(&name, Some(&a))
            .unwrap();
        assert_eq!(
            r.applicability() == HtmlAttributeApplicability::Applicable,
            applicable
        );
        assert_eq!(r.intrinsic().is_some(), applicable);
        assert_eq!(r.raw_attribute(), Some(&a));
    }
    for name in [
        QualifiedName::new(Namespace::none(), None, "HTTP-EQUIV"),
        QualifiedName::new(Namespace::from_uri("urn:x"), None, "http-equiv"),
    ] {
        let a = Attribute::new(name, "refresh");
        let r = HtmlValueInterpreter::new(7)
            .meta_http_equiv(&element(), Some(&a))
            .unwrap();
        assert_eq!(
            r.applicability(),
            HtmlAttributeApplicability::DifferentAttribute
        );
        assert_eq!(r.intrinsic(), None);
    }
}
#[test]
fn original_raw_cap_precedes_even_unapplied_role_and_preserves_utf16() {
    let a = raw(DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]));
    let interpreter = HtmlValueInterpreter::new(3);
    let r = interpreter.meta_http_equiv(&element(), Some(&a)).unwrap();
    assert_eq!(r.raw().unwrap().code_units(), &[0xd800, 0, 0xdc00]);
    assert_eq!(r.intrinsic(), None);
    for name in [element(), KnownHtmlElement::Div.qualified_name()] {
        assert!(
            HtmlValueInterpreter::new(2)
                .meta_http_equiv(&name, Some(&a))
                .is_err()
        );
    }
}
#[test]
fn local_metadata_identifies_actual_pragma_rule_without_claiming_document_effects() {
    let a = raw("refresh");
    let r = HtmlValueInterpreter::new(7)
        .meta_http_equiv(&element(), Some(&a))
        .unwrap();
    assert_eq!(
        r.source_clause(),
        "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-meta-http-equiv"
    );
}
