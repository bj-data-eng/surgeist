#![forbid(unsafe_code)]
//! Selectors 4 (2026-01-22), sections 7.1–7.2: checked authored arguments.
use surgeist_css::{
    CssComponentValueErrorKind, CssDirectionality, CssEmptyLanguageRangeList, CssLanguageRange,
    CssLanguageRangeKind, CssLanguageRangeList, CssNamespaceContext, CssPseudoClass, CssSelector,
    CssValueOrigin, parse_selector,
};

#[test]
fn decoded_identifiers_escape_without_restricting_direction_or_language_matching() {
    for (decoded, css) in [
        ("LTR", "LTR"),
        ("future", "future"),
        ("*-Latn", r"\*-Latn"),
        ("en US", r"en\ US"),
        ("1en", r"\31 en"),
    ] {
        let direction = CssDirectionality::try_new(decoded).unwrap();
        let language = CssLanguageRange::try_ident(decoded).unwrap();
        assert_eq!(direction.as_str(), decoded);
        assert_eq!(direction.to_css_string(), css);
        assert_eq!(language.as_str(), decoded);
        assert_eq!(language.kind(), CssLanguageRangeKind::Identifier);
        assert_eq!(language.to_css_string(), css);
        assert!(matches!(direction.origin(), CssValueOrigin::Programmatic));
        assert!(matches!(language.origin(), CssValueOrigin::Programmatic));
        for source in [format!(":dir({css})"), format!(":lang({css})")] {
            let parsed = parse_selector(&source, &CssNamespaceContext::default());
            assert!(parsed.is_clean(), "{source}: {parsed:?}");
            match parsed.syntax() {
                Some(CssSelector::PseudoClass(CssPseudoClass::Dir(value))) => {
                    assert_eq!(value.as_str(), decoded)
                }
                Some(CssSelector::PseudoClass(CssPseudoClass::Lang(list))) => {
                    assert_eq!(list.ranges().len(), 1);
                    assert_eq!(list.ranges()[0].as_str(), decoded);
                    assert_eq!(list.ranges()[0].kind(), CssLanguageRangeKind::Identifier);
                }
                other => panic!("linguistic pseudo: {other:?}"),
            }
        }
    }
}

#[test]
fn strings_preserve_empty_space_and_punctuation_with_canonical_quotes() {
    for (decoded, css) in [
        ("", "\"\""),
        ("en US", "\"en US\""),
        ("*-Latn", "\"*-Latn\""),
        ("EN", "\"EN\""),
        ("a\"b", "\"a\\\"b\""),
    ] {
        let range = CssLanguageRange::try_string(decoded).unwrap();
        assert_eq!(range.kind(), CssLanguageRangeKind::String);
        assert_eq!(range.as_str(), decoded);
        assert_eq!(range.to_css_string(), css);
        assert!(matches!(range.origin(), CssValueOrigin::Programmatic));
        let source = format!(":lang({css})");
        let parsed = parse_selector(&source, &CssNamespaceContext::default());
        assert!(parsed.is_clean(), "{source}: {parsed:?}");
        let Some(CssSelector::PseudoClass(CssPseudoClass::Lang(list))) = parsed.syntax() else {
            panic!("language list")
        };
        assert_eq!(list.ranges().len(), 1);
        assert_eq!(list.ranges()[0].as_str(), decoded);
        assert_eq!(list.ranges()[0].kind(), CssLanguageRangeKind::String);
    }
}

#[test]
fn constructed_nul_and_empty_identifiers_fail_without_fabricated_origins() {
    for decoded in ["", "en\0US"] {
        for error in [
            CssDirectionality::try_new(decoded).unwrap_err(),
            CssLanguageRange::try_ident(decoded).unwrap_err(),
        ] {
            assert_eq!(error.kind(), CssComponentValueErrorKind::InvalidIdentifier);
            assert!(matches!(error.origin(), CssValueOrigin::Programmatic));
        }
    }
    let error = CssLanguageRange::try_string("\0").unwrap_err();
    assert_eq!(error.kind(), CssComponentValueErrorKind::InvalidString);
    assert!(matches!(error.origin(), CssValueOrigin::Programmatic));
}

#[test]
fn lists_are_nonempty_and_preserve_order_duplicates_and_token_forms() {
    assert_eq!(
        CssLanguageRangeList::try_new(vec![]).unwrap_err(),
        CssEmptyLanguageRangeList
    );
    let ident = CssLanguageRange::try_ident("EN").unwrap();
    let string = CssLanguageRange::try_string("EN").unwrap();
    let list =
        CssLanguageRangeList::try_new(vec![ident.clone(), string.clone(), ident.clone()]).unwrap();
    assert_eq!(list.ranges(), [ident.clone(), string, ident.clone()]);
    assert_eq!(list.to_css_string(), "EN, \"EN\", EN");
    assert_eq!(
        CssLanguageRangeList::single(ident.clone()).ranges(),
        [ident]
    );
}

#[test]
fn parsed_tokens_keep_original_spans_and_canonicalize_only_when_requested() {
    let source = r#"/*😀*/:lang( \65 n , '\65 n', EN )"#;
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(report.is_clean(), "{report:?}");
    let Some(CssSelector::PseudoClass(CssPseudoClass::Lang(list))) = report.syntax() else {
        panic!("language list")
    };
    assert_eq!(list.to_css_string(), "en, \"en\", EN");
    let expected = [
        (r"\65 n", "en", CssLanguageRangeKind::Identifier),
        (r"'\65 n'", "en", CssLanguageRangeKind::String),
        ("EN", "EN", CssLanguageRangeKind::Identifier),
    ];
    for (range, (spelling, decoded, kind)) in list.ranges().iter().zip(expected) {
        assert_eq!(range.as_str(), decoded);
        assert_eq!(range.kind(), kind);
        let CssValueOrigin::Parsed(origin) = range.origin() else {
            panic!("parsed token origin")
        };
        let start = origin.span().start().byte_offset().value();
        let end = origin.span().end().byte_offset().value();
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(&source[start..end], spelling);
        assert_eq!(
            origin.span().start().column().value(),
            u32::try_from(source[..start].encode_utf16().count()).unwrap()
        );
    }
    assert_ne!(list.ranges()[0], CssLanguageRange::try_ident("en").unwrap());
    let direction_source = r":dir(\4c TR)";
    let report = parse_selector(direction_source, &CssNamespaceContext::default());
    let Some(CssSelector::PseudoClass(CssPseudoClass::Dir(direction))) = report.syntax() else {
        panic!("direction token")
    };
    assert_eq!(direction.as_str(), "LTR");
    assert_eq!(direction.to_css_string(), "LTR");
    let CssValueOrigin::Parsed(origin) = direction.origin() else {
        panic!("parsed direction")
    };
    assert_eq!(
        &direction_source[origin.span().start().byte_offset().value()
            ..origin.span().end().byte_offset().value()],
        r"\4c TR"
    );
}

#[test]
fn decoded_controls_serialize_and_reparse_as_single_tokens() {
    for (decoded, identifier_css, string_css) in [
        ("a\nb", "a\\a b", "\"a\\a b\""),
        ("a\u{1}b", "a\\1 b", "\"a\\1 b\""),
        ("a b", "a\\ b", "\"a b\""),
    ] {
        for (range, expected) in [
            (
                CssLanguageRange::try_ident(decoded).unwrap(),
                identifier_css,
            ),
            (CssLanguageRange::try_string(decoded).unwrap(), string_css),
        ] {
            assert_eq!(range.to_css_string(), expected);
            let source = format!(":lang({expected})");
            let report = parse_selector(&source, &CssNamespaceContext::default());
            assert!(report.is_clean(), "{source}: {report:?}");
            let Some(CssSelector::PseudoClass(CssPseudoClass::Lang(list))) = report.syntax() else {
                panic!("language argument")
            };
            assert_eq!(list.ranges()[0].as_str(), decoded);
            assert_eq!(list.ranges()[0].kind(), range.kind());
        }
    }
}

#[test]
fn source_nul_is_tokenizer_replacement_while_construction_rejects_nul() {
    for source in [
        ":lang(\"\0\")",
        r#":lang("\0 ")"#,
        ":lang(\0)",
        r":dir(\0 )",
    ] {
        let report = parse_selector(source, &CssNamespaceContext::default());
        assert!(report.is_clean(), "{source:?}: {report:?}");
        let (value, origin) = match report.syntax() {
            Some(CssSelector::PseudoClass(CssPseudoClass::Lang(list))) => {
                (list.ranges()[0].as_str(), list.ranges()[0].origin())
            }
            Some(CssSelector::PseudoClass(CssPseudoClass::Dir(direction))) => {
                (direction.as_str(), direction.origin())
            }
            other => panic!("linguistic argument: {other:?}"),
        };
        assert_eq!(value, "\u{fffd}");
        let CssValueOrigin::Parsed(origin) = origin else {
            panic!("parsed origin")
        };
        assert_eq!(origin.source().as_str(), source);
    }
}

#[test]
fn retained_linguistic_arguments_keep_the_original_eof_closure_contract() {
    // The recovery scanner records the unclosed function delimiter once.
    // A quoted token ends at EOF; its missing quote is not another block opener.
    for (source, expected) in [
        (":dir(ltr", "ltr"),
        (":lang(en", "en"),
        (":lang(\"en", "en"),
        ("/*😀*/\r\n:lang(\"en", "en"),
    ] {
        let report = parse_selector(source, &CssNamespaceContext::default());
        let value = match report.syntax() {
            Some(CssSelector::PseudoClass(CssPseudoClass::Dir(direction))) => direction.as_str(),
            Some(CssSelector::PseudoClass(CssPseudoClass::Lang(list))) => list.ranges()[0].as_str(),
            other => panic!("retained argument: {source}: {other:?}"),
        };
        assert_eq!(value, expected);
        assert!(!report.is_clean());
        let [closure] = report.diagnostics() else {
            panic!("one retained function closure: {report:?}")
        };
        assert_eq!(
            closure.action(),
            surgeist_css::CssRecoveryAction::RetainWithImplicitClosure
        );
        assert_eq!(
            closure.error().position().byte_offset().value(),
            source.len()
        );
        assert_eq!(closure.span().start().byte_offset().value(), source.len());
        assert_eq!(closure.span().end().byte_offset().value(), source.len());
        if source.contains('\n') {
            assert_eq!(closure.error().position().line().value(), 1);
            assert_eq!(closure.error().position().column().value(), 9);
        }
        assert!(report.into_validation_result().is_err());
    }
}

#[test]
fn actual_bad_strings_reject_instead_of_claiming_implicit_retention() {
    for source in [":lang(\"en\n)", ":lang('en\r\n)"] {
        let report = parse_selector(source, &CssNamespaceContext::default());
        assert!(report.syntax().is_none(), "{source:?}: {report:?}");
        assert!(!report.is_clean());
        let [diagnostic] = report.diagnostics() else {
            panic!("whole input rejection")
        };
        assert_eq!(
            diagnostic.error().code(),
            surgeist_css::CssErrorCode::InvalidSelector
        );
        assert_eq!(
            diagnostic.action(),
            surgeist_css::CssRecoveryAction::RejectInput
        );
    }
}
