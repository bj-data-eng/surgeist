//! Reserved font-name grammar from the selected published standards:
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-family-name-syntax
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-face-src-parsing
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#custom-idents
//! The reserved default identifier is excluded in every ASCII case permutation.
//! Quoted strings and decoded names retain their distinct construction semantics.

#![forbid(unsafe_code)]

use surgeist_css::{
    CssErrorCode, CssFontFaceFamily, CssFontFaceSource, CssFontFamilyName, CssFontFamilyNameKind,
    CssFontLocalName, CssFontValue, CssKnownProperty, CssKnownPropertyValueRef, CssRecoveryAction,
    CssRule, parse_sheet, parse_style_attribute, validate_sheet, validate_style_attribute,
};

// These are authored identifier sequences. Escapes are decoded by the real
// parser before the reserved-token check; whitespace terminating a hex escape
// is not a separator between identifiers.
const RESERVED_NAME_SYNTAX: &[&str] = &[
    "default",
    "DEFAULT",
    "DeFaUlT",
    "default A",
    "A default",
    "A default B",
    r"\64 efault",
    r"A \64 efault B",
    r"de\66 ault A",
];

#[test]
fn font_family_property_and_shorthand_reject_reserved_default_tokens() {
    for name in RESERVED_NAME_SYNTAX {
        for declaration in [format!("font-family:{name}"), format!("font:16px {name}")] {
            let source = format!("{declaration};color:red");
            let report = parse_style_attribute(&source);
            assert_eq!(
                report.syntax().len(),
                1,
                "invalid family must discard its declaration: {source}; {report:?}"
            );
            assert_eq!(
                report.syntax()[0].known().unwrap().property(),
                CssKnownProperty::Color
            );
            let [diagnostic] = report.diagnostics() else {
                panic!("expected one invalid family declaration: {source}; {report:?}");
            };
            assert_eq!(
                diagnostic.error().code(),
                CssErrorCode::InvalidPropertyValue
            );
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
            assert_eq!(
                validate_style_attribute(&source)
                    .expect_err("validation must reject the recovered declaration")
                    .diagnostics(),
                report.diagnostics()
            );
        }
    }
}

#[test]
fn font_face_family_descriptor_rejects_reserved_default_tokens() {
    for name in RESERVED_NAME_SYNTAX {
        let source = format!("@font-face{{font-family:{name};src:url(fallback.woff2)}}");
        let report = parse_sheet(&source);
        let [CssRule::FontFace(rule)] = report.syntax().rules() else {
            panic!("family rejection must retain its outer rule: {source}; {report:?}");
        };
        assert!(
            rule.descriptors().font_family().is_none(),
            "reserved family must be absent: {source}; {report:?}"
        );
        let [CssFontFaceSource::Url(url)] = rule.descriptors().src().unwrap().sources() else {
            panic!("unrelated source descriptor must survive: {source}; {report:?}");
        };
        assert_eq!(url.url(), "fallback.woff2");
        let [diagnostic] = report.diagnostics() else {
            panic!("expected one invalid family descriptor: {source}; {report:?}");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
        assert_eq!(
            validate_sheet(&source)
                .expect_err("validation must reject the recovered family descriptor")
                .diagnostics(),
            report.diagnostics()
        );
    }
}

#[test]
fn local_font_source_rejects_reserved_default_tokens() {
    for name in RESERVED_NAME_SYNTAX {
        let source = format!("@font-face{{font-family:Demo;src:local({name})}}");
        let report = parse_sheet(&source);
        let [CssRule::FontFace(rule)] = report.syntax().rules() else {
            panic!("source rejection must retain its outer rule: {source}; {report:?}");
        };
        assert_eq!(rule.descriptors().font_family().unwrap().as_str(), "Demo");
        assert!(
            rule.descriptors().src().is_none(),
            "an invalid sole local source must leave no src: {source}; {report:?}"
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("expected one invalid source descriptor: {source}; {report:?}");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
        assert_eq!(
            validate_sheet(&source)
                .expect_err("validation must reject the recovered source descriptor")
                .diagnostics(),
            report.diagnostics()
        );
    }
}

#[test]
fn quoted_reserved_names_and_nonreserved_identifiers_preserve_decoded_names() {
    use CssFontFamilyNameKind::{IdentSequence, Quoted};

    for (authored, decoded, kind) in [
        (r#""default""#, "default", Quoted),
        (r#"'DeFaUlT'"#, "DeFaUlT", Quoted),
        (r#""A default B""#, "A default B", Quoted),
        (r#""\64 efault""#, "default", Quoted),
        ("Defaultish", "Defaultish", IdentSequence),
        ("My-default", "My-default", IdentSequence),
        (r"default\41", "defaultA", IdentSequence),
        (r"A\ default", "A default", IdentSequence),
        (r"default\ A", "default A", IdentSequence),
        ("A\u{a0}default", "A\u{a0}default", IdentSequence),
    ] {
        let style = format!("font-family:{authored};font:16px {authored}");
        let report = parse_style_attribute(&style);
        assert!(report.is_clean(), "{style}; {report:?}");
        assert_eq!(report.syntax().len(), 2);
        for declaration in report.syntax().iter() {
            let families = match declaration.known().unwrap().property_value().unwrap() {
                CssKnownPropertyValueRef::FontFamily(value) => value.families(),
                CssKnownPropertyValueRef::Font(value) => {
                    let CssFontValue::Explicit(value) = value.font() else {
                        panic!("expected explicit font shorthand: {style}");
                    };
                    value.families()
                }
                other => panic!("expected font family or shorthand: {other:?}"),
            };
            let [family] = families.families() else {
                panic!("expected one preserved name: {style}");
            };
            assert_eq!(family.kind(), kind, "{style}");
            assert_eq!(family.as_str(), decoded, "{style}");
        }

        let sheet = format!("@font-face{{font-family:{authored};src:local({authored})}}");
        let report = parse_sheet(&sheet);
        assert!(report.is_clean(), "{sheet}; {report:?}");
        let [CssRule::FontFace(rule)] = report.syntax().rules() else {
            panic!("expected font-face: {sheet}");
        };
        assert_eq!(rule.descriptors().font_family().unwrap().as_str(), decoded);
        let [CssFontFaceSource::Local(local)] = rule.descriptors().src().unwrap().sources() else {
            panic!("expected one local source: {sheet}");
        };
        assert_eq!(local.as_str(), decoded);
    }
}

#[test]
fn checked_identifier_names_reject_reserved_default_as_the_whole_decoded_name() {
    for name in ["default", "DEFAULT", "DeFaUlT"] {
        assert_eq!(
            CssFontFamilyName::try_ident_sequence(name),
            None,
            "escaping an identifier cannot make the decoded reserved name valid: {name}"
        );
    }
}

#[test]
fn checked_quoted_and_decoded_face_names_preserve_reserved_spelling() {
    for name in ["default", "DEFAULT", "DeFaUlT", "A default", "default A"] {
        let quoted = CssFontFamilyName::try_quoted(name).expect("valid quoted font name");
        assert_eq!(quoted.kind(), CssFontFamilyNameKind::Quoted);
        assert_eq!(quoted.as_str(), name);

        // These wrappers store decoded names, without an unquoted-token kind.
        // They must retain names obtained from valid quoted authored syntax.
        assert_eq!(CssFontFaceFamily::try_new(name).unwrap().as_str(), name);
        assert_eq!(CssFontLocalName::try_new(name).unwrap().as_str(), name);
    }

    // An escaped space can occur inside one identifier. Once a name has been
    // decoded, splitting it again would invent token boundaries that were lost.
    for name in ["A default", "default A", "My-default", "A\u{a0}default"] {
        let sequence = CssFontFamilyName::try_ident_sequence(name)
            .expect("decoded nonreserved identifier name");
        assert_eq!(sequence.kind(), CssFontFamilyNameKind::IdentSequence);
        assert_eq!(sequence.as_str(), name);
    }
}
