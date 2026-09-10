#![forbid(unsafe_code)]

//! Authored font-family grammar from the selected published standards:
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-family-name-syntax
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#generic-family-name-syntax
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#local-font-fallback
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#custom-idents
//! https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#input-preprocessing
//! Fonts4 section 2.7 limits system-font keyword meaning to the initial position:
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-prop
//! Its explicit family-name rule and property-specific exclusions govern literal
//! names; the broader informative quoting note in section 2.1.1 cannot override them.
//! Recovery, exact authored text, and importance are Surgeist contracts.

use surgeist_css::{
    CssDeclaration, CssErrorCode, CssFontFaceSource, CssFontFamilyList, CssFontFamilyNameKind,
    CssFontValue, CssImportance, CssKnownProperty, CssKnownPropertyValueRef, CssRecoveryAction,
    CssRule, CssSystemFont, parse_sheet, parse_style_attribute, validate_sheet,
    validate_style_attribute,
};

const SIMPLE_GENERICS: &[&str] = &[
    "serif",
    "sans-serif",
    "cursive",
    "fantasy",
    "monospace",
    "system-ui",
    "math",
    "ui-serif",
    "ui-sans-serif",
    "ui-monospace",
    "ui-rounded",
];

const FUNCTION_GENERICS: &[&str] = &[
    "generic(fangsong)",
    "generic(kai)",
    "generic(khmer-mul)",
    "generic(nastaliq)",
];

const RESERVED_IDENTIFIERS: &[&str] = &[
    "serif",
    "sans-serif",
    "cursive",
    "fantasy",
    "monospace",
    "system-ui",
    "math",
    "ui-serif",
    "ui-sans-serif",
    "ui-monospace",
    "ui-rounded",
    "initial",
    "inherit",
    "unset",
    "revert",
    "revert-layer",
    "default",
];

fn families(declaration: &CssDeclaration) -> &CssFontFamilyList {
    match declaration.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::FontFamily(value) => value.families(),
        CssKnownPropertyValueRef::Font(value) => {
            let CssFontValue::Explicit(font) = value.font() else {
                panic!("expected an explicit font shorthand");
            };
            font.families()
        }
        other => panic!("expected font-family or font, got {other:?}"),
    }
}

fn assert_literal_in_all_contexts(authored: &str, decoded: &str, kind: CssFontFamilyNameKind) {
    let source = format!("font-family:{authored};font:16px {authored}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2, "{source}");
    for declaration in report.syntax().iter() {
        let [name] = families(declaration).families() else {
            panic!("expected one literal name: {source}");
        };
        assert_eq!(name.kind(), kind, "{source}");
        assert_eq!(name.as_str(), decoded, "{source}");
        assert_eq!(name.generic_family(), None, "{source}");
    }
    validate_style_attribute(&source).expect("clean properties must validate");

    let source = format!("@font-face{{font-family:{authored};src:local({authored})}}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("expected one font face: {source}");
    };
    assert_eq!(face.descriptors().font_family().unwrap().as_str(), decoded);
    let [CssFontFaceSource::Local(name)] = face.descriptors().src().unwrap().sources() else {
        panic!("expected one local source: {source}");
    };
    assert_eq!(name.as_str(), decoded, "{source}");
    validate_sheet(&source).expect("clean descriptors must validate");
}

fn assert_property_rejection(authored: &str) {
    for declaration in [
        format!("font-family:{authored}"),
        format!("font:16px {authored}"),
    ] {
        let source = format!("{declaration};color:red");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "{source}: {report:?}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("expected one dropped declaration: {source}: {report:?}");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(
            validate_style_attribute(&source).unwrap_err().diagnostics(),
            report.diagnostics(),
        );
    }
}

#[test]
fn all_fifteen_generics_are_recognized_in_properties_and_explicit_font_tails() {
    for canonical in SIMPLE_GENERICS.iter().chain(FUNCTION_GENERICS) {
        for authored in [canonical.to_string(), canonical.to_ascii_uppercase()] {
            let source = format!("font-family:{authored};font:16px {authored}");
            let report = parse_style_attribute(&source);
            assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
            assert_eq!(report.syntax().len(), 2);
            for declaration in report.syntax().iter() {
                let [family] = families(declaration).families() else {
                    panic!("expected one generic family: {source}");
                };
                assert_eq!(family.kind(), CssFontFamilyNameKind::Generic, "{source}");
                assert!(family.generic_family().is_some(), "{source}");
                assert_eq!(family.as_str(), *canonical, "{source}");
            }
        }
    }
}

#[test]
fn escaped_generic_keywords_and_function_arguments_have_the_same_meaning() {
    for (authored, canonical) in [
        (r"\73 erif", "serif"),
        (r"\53 YSTEM-ui", "system-ui"),
        (r"ui-\72 ounded", "ui-rounded"),
        (r"g\65 neric(\66 angsong)", "generic(fangsong)"),
        (r"GENERIC(\4b AI)", "generic(kai)"),
        (
            r"generic(/*before*/Khmer-Mul/*after*/)",
            "generic(khmer-mul)",
        ),
        (r"generic(nastali\71)", "generic(nastaliq)"),
    ] {
        let source = format!("font-family:{authored}");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let CssKnownPropertyValueRef::FontFamily(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("expected font-family");
        };
        assert_eq!(value.as_css(), authored);
        let [family] = value.families().families() else {
            panic!("expected one generic: {source}");
        };
        assert!(family.generic_family().is_some(), "{source}");
        assert_eq!(family.as_str(), canonical, "{source}");
    }
}

#[test]
fn generic_lists_preserve_authored_order_source_and_importance() {
    let authored = r#""Avenir Next", SYSTEM-UI, g\65 neric(kai), serif"#;
    let source = format!("  FONT-FAMILY: {authored} ! IMPORTANT; color:red");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);
    let declaration = &report.syntax()[0];
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert_eq!(declaration.position().unwrap().byte_offset().value(), 2);
    let CssKnownPropertyValueRef::FontFamily(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("expected font-family");
    };
    assert_eq!(value.as_css(), authored);
    assert_eq!(
        value
            .families()
            .families()
            .iter()
            .map(|name| name.as_str())
            .collect::<Vec<_>>(),
        ["Avenir Next", "system-ui", "generic(kai)", "serif"],
    );
    assert_eq!(value.families().families()[0].generic_family(), None);
    assert!(
        value.families().families()[1..]
            .iter()
            .all(|name| name.generic_family().is_some())
    );
    assert_eq!(report.syntax()[1].importance(), CssImportance::Normal);
}

#[test]
fn empty_and_whitespace_names_are_valid_authored_literals() {
    for (authored, decoded, kind) in [
        (r#""""#, "", CssFontFamilyNameKind::Quoted),
        ("''", "", CssFontFamilyNameKind::Quoted),
        (r#"" ""#, " ", CssFontFamilyNameKind::Quoted),
        (r#""\9""#, "\t", CssFontFamilyNameKind::Quoted),
        (r"\20", " ", CssFontFamilyNameKind::IdentSequence),
        (r"\9", "\t", CssFontFamilyNameKind::IdentSequence),
    ] {
        assert_literal_in_all_contexts(authored, decoded, kind);
    }
}

#[test]
fn reserved_identifiers_are_rejected_per_decoded_token_not_per_joined_word() {
    for keyword in RESERVED_IDENTIFIERS {
        for token in [keyword.to_string(), keyword.to_ascii_uppercase()] {
            for authored in [
                format!("A {token}"),
                format!("{token} A"),
                format!("A {token} B"),
            ] {
                assert_property_rejection(&authored);
            }
        }
        assert_literal_in_all_contexts(
            &format!("A\\ {keyword}"),
            &format!("A {keyword}"),
            CssFontFamilyNameKind::IdentSequence,
        );
        assert_literal_in_all_contexts(
            &format!("\"{keyword}\""),
            keyword,
            CssFontFamilyNameKind::Quoted,
        );
    }
    for authored in [r"A \73 ystem-ui", r"A \64 efault", r"revert-\6c ayer A"] {
        assert_property_rejection(authored);
    }
}

#[test]
fn bare_function_names_and_nonreserved_font_words_remain_literal_names() {
    for name in [
        "generic",
        "fangsong",
        "kai",
        "khmer-mul",
        "nastaliq",
        "emoji",
        "bold",
        "normal",
        "auto",
        "span",
        "revert-rule",
    ] {
        assert_literal_in_all_contexts(name, name, CssFontFamilyNameKind::IdentSequence);
    }
    for name in FUNCTION_GENERICS {
        assert_literal_in_all_contexts(&format!("\"{name}\""), name, CssFontFamilyNameKind::Quoted);
    }
    for (authored, decoded) in [
        (r"serif\ A", "serif A"),
        (r"default\ A", "default A"),
        ("A\u{a0}default", "A\u{a0}default"),
        (r"\31 st", "1st"),
        (r"A\,B", "A,B"),
    ] {
        assert_literal_in_all_contexts(authored, decoded, CssFontFamilyNameKind::IdentSequence);
    }
}

#[test]
fn system_font_spellings_are_literal_names_in_family_positions() {
    for name in [
        "caption",
        "icon",
        "menu",
        "message-box",
        "small-caption",
        "status-bar",
    ] {
        for spelling in [name.to_owned(), name.to_ascii_uppercase()] {
            for literal in [
                spelling.clone(),
                format!("A {spelling}"),
                format!("{spelling} A"),
                format!("A {spelling} B"),
            ] {
                assert_literal_in_all_contexts(
                    &literal,
                    &literal,
                    CssFontFamilyNameKind::IdentSequence,
                );
            }
            assert_literal_in_all_contexts(
                &format!("\"{spelling}\""),
                &spelling,
                CssFontFamilyNameKind::Quoted,
            );
        }
    }
    for (authored, decoded) in [
        (r"\6d enu", "menu"),
        (r"M\45 NU", "MENU"),
        (r"A \6d enu", "A menu"),
        (r"\6d enu A", "menu A"),
    ] {
        assert_literal_in_all_contexts(authored, decoded, CssFontFamilyNameKind::IdentSequence);
    }
    // This is the normative section 2.7 example, with its explicit size retained.
    let report = parse_style_attribute("font:large menu");
    assert!(report.is_clean(), "{report:?}");
    let CssKnownPropertyValueRef::Font(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected font shorthand");
    };
    let CssFontValue::Explicit(font) = value.font() else {
        panic!("noninitial menu must be a literal family name");
    };
    assert_eq!(font.size(), &surgeist_css::CssFontSize::Large);
    let [name] = font.families().families() else {
        panic!("expected exactly the menu family");
    };
    assert_eq!(name.as_str(), "menu");
    assert_eq!(name.kind(), CssFontFamilyNameKind::IdentSequence);
    assert_eq!(name.generic_family(), None);
}

#[test]
fn malformed_generics_and_empty_family_items_drop_only_their_declaration() {
    for authored in [
        "generic()",
        "generic(unknown)",
        "generic(serif)",
        "generic(kai nastaliq)",
        "generic(kai,nastaliq)",
        "generic(\"kai\")",
        "generic (kai)",
        "generic(kai) A",
        "A generic(kai)",
        "serif A",
        "",
        ",serif",
        "serif,",
        "A,,B",
        "\"A\" B",
    ] {
        assert_property_rejection(authored);
    }
}

#[test]
fn font_face_and_local_names_reject_generics_and_reserved_tokens_with_local_recovery() {
    for authored in RESERVED_IDENTIFIERS.iter().chain(FUNCTION_GENERICS) {
        let source = format!(
            "@font-face{{font-family:{authored};src:url(before),local({authored}),local(After)}}.after{{color:red}}"
        );
        let report = parse_sheet(&source);
        let [CssRule::FontFace(face), CssRule::Style(_)] = report.syntax().rules() else {
            panic!("expected retained rules: {source}: {report:?}");
        };
        assert!(face.descriptors().font_family().is_none(), "{source}");
        let [
            CssFontFaceSource::Url(before),
            CssFontFaceSource::Local(after),
        ] = face.descriptors().src().unwrap().sources()
        else {
            panic!("expected both valid source fallbacks: {source}: {report:?}");
        };
        assert_eq!(before.url(), "before");
        assert_eq!(after.as_str(), "After");
        let [family_diagnostic, local_diagnostic] = report.diagnostics() else {
            panic!("expected two independent recoveries: {source}: {report:?}");
        };
        assert_eq!(
            family_diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        assert_eq!(
            family_diagnostic.action(),
            CssRecoveryAction::DropDescriptor
        );
        assert_eq!(
            local_diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        assert_eq!(
            local_diagnostic.action(),
            CssRecoveryAction::DropFontSourceListItem
        );
        let member = format!("local({authored})");
        let start = source.find(&member).unwrap();
        assert_eq!(local_diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            local_diagnostic.span().end().byte_offset().value(),
            start + member.len()
        );
        assert_eq!(
            validate_sheet(&source).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }
}

#[test]
fn css_nul_preprocessing_and_escape_decoding_preserve_replacement_characters() {
    for (authored, kind) in [
        ("\"A\0B\"", CssFontFamilyNameKind::Quoted),
        ("A\0B", CssFontFamilyNameKind::IdentSequence),
        (r#""A\0 B""#, CssFontFamilyNameKind::Quoted),
        (r"A\0 B", CssFontFamilyNameKind::IdentSequence),
    ] {
        assert_literal_in_all_contexts(authored, "A\u{fffd}B", kind);
    }
}

#[test]
fn global_family_values_and_whole_system_font_shorthands_keep_their_branches() {
    for keyword in ["initial", "inherit", "unset", "revert", "revert-layer"] {
        let source = format!("font-family:{keyword}");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {report:?}");
        assert!(report.syntax()[0].known().unwrap().global().is_some());
        assert_property_rejection(&format!("{keyword},serif"));
    }
    for (spelling, expected) in [
        ("caption", CssSystemFont::Caption),
        ("icon", CssSystemFont::Icon),
        ("menu", CssSystemFont::Menu),
        ("message-box", CssSystemFont::MessageBox),
        ("small-caption", CssSystemFont::SmallCaption),
        ("status-bar", CssSystemFont::StatusBar),
        (r"\6d enu", CssSystemFont::Menu),
    ] {
        for authored in [spelling.to_owned(), spelling.to_ascii_uppercase()] {
            let source = format!("font:{authored}");
            let report = parse_style_attribute(&source);
            assert!(report.is_clean(), "{source}: {report:?}");
            let CssKnownPropertyValueRef::Font(value) = report.syntax()[0]
                .known()
                .unwrap()
                .property_value()
                .unwrap()
            else {
                panic!("expected font shorthand");
            };
            assert_eq!(value.font(), &CssFontValue::System(expected), "{source}");
        }
    }
}
