#![forbid(unsafe_code)]

//! Checked construction preserves decoded identifier boundaries and font-family meaning.
//! Name and generic grammar:
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-family-name-syntax
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#generic-family-name-syntax
//! Excluded custom identifiers:
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#custom-idents
//! NUL rejection prevents the identity change imposed by CSS serialization:
//! https://www.w3.org/TR/2021/WD-cssom-1-20210826/#common-serializing-idioms
//! Constructors take decoded content, never CSS source to tokenize a second time.
//! Fonts4 section 2.7 explicitly treats system spellings as family names outside
//! the initial shorthand position; this and the property-specific name grammar
//! govern the broader informative quoting note in section 2.1.1:
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-prop

use surgeist_css::{
    CssExplicitFont, CssFontFaceFamily, CssFontFamilyList, CssFontFamilyName,
    CssFontFamilyNameKind, CssFontLocalName, CssFontSize, CssFontValue, CssGenericFontFamily,
    CssImportance, CssKnownPropertyValueRef, CssSystemFont, parse_style_attribute,
};

fn parsed_name(authored: &str) -> CssFontFamilyName {
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
    let [name] = value.families().families() else {
        panic!("expected one family name: {source}");
    };
    name.clone()
}

fn decoded_identifier_boundaries() {
    for (authored, tokens, joined) in [
        (r"A\ default", vec!["A default"], "A default"),
        (r"default\ A", vec!["default A"], "default A"),
        (r"A\ B C", vec!["A B", "C"], "A B C"),
        ("A B C", vec!["A", "B", "C"], "A B C"),
        (r"\20", vec![" "], " "),
        (r"\9", vec!["\t"], "\t"),
        (r"A\,B", vec!["A,B"], "A,B"),
        (r"\31 st", vec!["1st"], "1st"),
        ("A\u{a0}default", vec!["A\u{a0}default"], "A\u{a0}default"),
    ] {
        let owned_tokens = tokens
            .iter()
            .map(|token| (*token).to_owned())
            .collect::<Vec<_>>();
        let constructed = CssFontFamilyName::try_ident_sequence(owned_tokens.clone())
            .expect("valid decoded identifier tokens");
        assert_eq!(constructed.kind(), CssFontFamilyNameKind::IdentSequence);
        assert_eq!(
            constructed.identifier_tokens(),
            Some(owned_tokens.as_slice())
        );
        assert_eq!(constructed.as_str(), joined);
        assert_eq!(constructed.generic_family(), None);
        let parsed = parsed_name(authored);
        assert_eq!(parsed.identifier_tokens(), Some(owned_tokens.as_slice()));
        assert_eq!(parsed.as_str(), joined);
        assert_eq!(parsed, constructed);
    }

    let one = CssFontFamilyName::try_ident("A B").unwrap();
    let two = CssFontFamilyName::try_ident_sequence(vec!["A".into(), "B".into()]).unwrap();
    assert_eq!(one.as_str(), two.as_str());
    assert_eq!(one.identifier_tokens().unwrap(), &["A B".to_owned()]);
    assert_eq!(
        two.identifier_tokens().unwrap(),
        &["A".to_owned(), "B".to_owned()]
    );
    assert_ne!(one, two, "authored equality includes identifier boundaries");

    let decoded_backslash = CssFontFamilyName::try_ident(r"A\31 B").unwrap();
    assert_eq!(decoded_backslash.as_str(), r"A\31 B");
    assert_eq!(
        decoded_backslash.identifier_tokens().unwrap(),
        &[r"A\31 B".to_owned()]
    );
    println!("decoded identifier boundaries: ok");
}

fn token_exclusions_and_literal_strings() {
    let reserved = [
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
    for keyword in reserved {
        for token in [keyword.to_owned(), keyword.to_ascii_uppercase()] {
            assert!(CssFontFamilyName::try_ident(token.clone()).is_none());
            for tokens in [
                vec![token.clone()],
                vec!["A".to_owned(), token.clone()],
                vec![token.clone(), "A".to_owned()],
                vec!["A".to_owned(), token.clone(), "B".to_owned()],
            ] {
                assert!(CssFontFamilyName::try_ident_sequence(tokens).is_none());
            }
            let joined_token = format!("A {token}");
            let escaped = CssFontFamilyName::try_ident(joined_token.clone()).unwrap();
            assert_eq!(escaped.identifier_tokens().unwrap(), &[joined_token]);
            let quoted = CssFontFamilyName::try_quoted(token.clone()).unwrap();
            assert_eq!(quoted.kind(), CssFontFamilyNameKind::Quoted);
            assert_eq!(quoted.as_str(), token);
            assert_eq!(quoted.generic_family(), None);
            assert_eq!(quoted.identifier_tokens(), None);
        }
    }
    for name in [
        "generic",
        "fangsong",
        "kai",
        "khmer-mul",
        "nastaliq",
        "emoji",
        "normal",
        "bold",
        "auto",
        "span",
        "revert-rule",
    ] {
        let literal = CssFontFamilyName::try_ident(name).unwrap();
        assert_eq!(literal.generic_family(), None);
        assert_eq!(literal.as_str(), name);
    }
    assert!(CssFontFamilyName::try_ident_sequence(Vec::new()).is_none());
    for tokens in [
        vec!["".into()],
        vec!["A".into(), "".into()],
        vec!["".into(), "A".into()],
    ] {
        assert!(CssFontFamilyName::try_ident_sequence(tokens).is_none());
    }
    assert!(CssFontFamilyName::try_ident("").is_none());

    for decoded in [
        "",
        " ",
        "\t",
        "\n",
        "\r",
        "\u{c}",
        "serif",
        "default",
        "generic(kai)",
        "A\\B",
        "A\u{fffd}B",
    ] {
        let quoted = CssFontFamilyName::try_quoted(decoded).unwrap();
        assert_eq!(quoted.kind(), CssFontFamilyNameKind::Quoted);
        assert_eq!(quoted.as_str(), decoded);
        assert_eq!(quoted.identifier_tokens(), None);
        let list = CssFontFamilyList::try_new(vec![quoted.clone()]).unwrap();
        assert_eq!(list.families(), &[quoted]);
        assert_eq!(
            CssFontFaceFamily::try_new(decoded).unwrap().as_str(),
            decoded
        );
        assert_eq!(
            CssFontLocalName::try_new(decoded).unwrap().as_str(),
            decoded
        );
    }
    assert!(CssFontFamilyList::try_new(Vec::new()).is_none());
    for decoded in ["\0", "A\0B", "\0 "] {
        assert!(CssFontFamilyName::try_quoted(decoded).is_none());
        assert!(CssFontFamilyName::try_ident(decoded).is_none());
        assert!(CssFontFamilyName::try_ident_sequence(vec![decoded.into()]).is_none());
        assert!(CssFontFamilyName::try_ident_sequence(vec!["A".into(), decoded.into()]).is_none());
        assert!(CssFontFaceFamily::try_new(decoded).is_none());
        assert!(CssFontLocalName::try_new(decoded).is_none());
    }
    println!("token exclusions and literal strings: ok");
}

fn typed_generic_families() {
    for (generic, canonical) in [
        (CssGenericFontFamily::Serif, "serif"),
        (CssGenericFontFamily::SansSerif, "sans-serif"),
        (CssGenericFontFamily::Cursive, "cursive"),
        (CssGenericFontFamily::Fantasy, "fantasy"),
        (CssGenericFontFamily::Monospace, "monospace"),
        (CssGenericFontFamily::SystemUi, "system-ui"),
        (CssGenericFontFamily::Math, "math"),
        (CssGenericFontFamily::UiSerif, "ui-serif"),
        (CssGenericFontFamily::UiSansSerif, "ui-sans-serif"),
        (CssGenericFontFamily::UiMonospace, "ui-monospace"),
        (CssGenericFontFamily::UiRounded, "ui-rounded"),
        (CssGenericFontFamily::Fangsong, "generic(fangsong)"),
        (CssGenericFontFamily::Kai, "generic(kai)"),
        (CssGenericFontFamily::KhmerMul, "generic(khmer-mul)"),
        (CssGenericFontFamily::Nastaliq, "generic(nastaliq)"),
    ] {
        let constructed = CssFontFamilyName::generic(generic);
        assert_eq!(constructed.generic_family(), Some(generic));
        assert_eq!(constructed.kind(), CssFontFamilyNameKind::Generic);
        assert_eq!(constructed.identifier_tokens(), None);
        assert_eq!(constructed.as_str(), canonical);
        for authored in [canonical.to_owned(), canonical.to_ascii_uppercase()] {
            let parsed = parsed_name(&authored);
            assert_eq!(parsed.generic_family(), Some(generic), "{authored}");
            assert_eq!(parsed.as_str(), canonical);
            assert_eq!(parsed, constructed);
        }
        let literal = CssFontFamilyName::try_quoted(canonical).unwrap();
        assert_eq!(literal.generic_family(), None);
        assert_ne!(
            constructed, literal,
            "a quoted name cannot acquire generic meaning"
        );
    }
    println!("typed generic families: ok");
}

fn system_spellings_in_literal_names_and_whole_shorthands() {
    for (name, system) in [
        ("caption", CssSystemFont::Caption),
        ("icon", CssSystemFont::Icon),
        ("menu", CssSystemFont::Menu),
        ("message-box", CssSystemFont::MessageBox),
        ("small-caption", CssSystemFont::SmallCaption),
        ("status-bar", CssSystemFont::StatusBar),
    ] {
        for spelling in [name.to_owned(), name.to_ascii_uppercase()] {
            let single = CssFontFamilyName::try_ident(spelling.clone()).unwrap();
            assert_eq!(single.identifier_tokens().unwrap(), &[spelling.clone()]);
            assert_eq!(single.as_str(), spelling);
            assert_eq!(single.generic_family(), None);
            assert_eq!(parsed_name(&spelling), single);
            for tokens in [
                vec![spelling.clone()],
                vec!["A".to_owned(), spelling.clone()],
                vec![spelling.clone(), "A".to_owned()],
                vec!["A".to_owned(), spelling.clone(), "B".to_owned()],
            ] {
                let joined = tokens.join(" ");
                let literal = CssFontFamilyName::try_ident_sequence(tokens.clone()).unwrap();
                assert_eq!(literal.identifier_tokens(), Some(tokens.as_slice()));
                assert_eq!(literal.as_str(), joined);
                assert_eq!(literal.generic_family(), None);
                assert_eq!(parsed_name(&joined), literal);
                let quoted = CssFontFamilyName::try_quoted(joined.clone()).unwrap();
                assert_eq!(quoted.kind(), CssFontFamilyNameKind::Quoted);
                assert_eq!(quoted.as_str(), joined);
                assert_eq!(quoted.generic_family(), None);
                assert_eq!(
                    CssFontFaceFamily::try_new(joined.clone()).unwrap().as_str(),
                    joined
                );
                assert_eq!(
                    CssFontLocalName::try_new(joined.clone()).unwrap().as_str(),
                    joined
                );
            }
            let source = format!("font:{spelling}");
            let report = parse_style_attribute(&source);
            assert!(report.is_clean(), "{source}: {report:?}");
            let CssKnownPropertyValueRef::Font(value) = report.syntax()[0]
                .known()
                .unwrap()
                .property_value()
                .unwrap()
            else {
                panic!("expected whole system-font shorthand");
            };
            assert_eq!(value.font(), &CssFontValue::System(system));
        }
    }
    for (authored, tokens) in [
        (r"\6d enu", vec!["menu".to_owned()]),
        (r"M\45 NU", vec!["MENU".to_owned()]),
        (r"A \6d enu", vec!["A".to_owned(), "menu".to_owned()]),
    ] {
        let parsed = parsed_name(authored);
        assert_eq!(parsed.identifier_tokens(), Some(tokens.as_slice()));
        assert_eq!(
            parsed,
            CssFontFamilyName::try_ident_sequence(tokens).unwrap()
        );
    }
    println!("system spellings retain contextual meaning: ok");
}

fn current_property_and_font_values() {
    let list = CssFontFamilyList::try_new(vec![
        CssFontFamilyName::try_quoted("").unwrap(),
        CssFontFamilyName::try_ident_sequence(vec!["A".into(), "B".into()]).unwrap(),
        CssFontFamilyName::generic(CssGenericFontFamily::Kai),
        CssFontFamilyName::generic(CssGenericFontFamily::SystemUi),
    ])
    .unwrap();
    let font = CssExplicitFont::try_new(
        None,
        None,
        None,
        None,
        CssFontSize::Medium,
        None,
        list.clone(),
    )
    .unwrap();
    assert_eq!(font.families(), &list);
    let authored_family = r#""", A B, GeNeRiC(KAI), SYSTEM-UI"#;
    let authored_font = format!("medium {authored_family}");
    let source = format!("font-family:{authored_family}!important;font:{authored_font}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);
    for declaration in report.syntax().iter() {
        assert_eq!(declaration.importance(), CssImportance::Important);
    }
    let CssKnownPropertyValueRef::FontFamily(family_value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected current font-family");
    };
    assert_eq!(family_value.families(), &list);
    assert_eq!(family_value.as_css(), authored_family);
    let CssKnownPropertyValueRef::Font(font_value) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected current font shorthand");
    };
    assert_eq!(font_value.font(), &CssFontValue::Explicit(font));
    assert_eq!(font_value.as_css(), authored_font);
    println!("current property and font values: ok");
}

fn main() {
    decoded_identifier_boundaries();
    token_exclusions_and_literal_strings();
    typed_generic_families();
    system_spellings_in_literal_names_and_whole_shorthands();
    current_property_and_font_values();
}
