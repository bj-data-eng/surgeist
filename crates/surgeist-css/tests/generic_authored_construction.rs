#![forbid(unsafe_code)]

use surgeist_css::*;

#[test]
fn decoded_empty_and_whitespace_targets_keep_their_authored_roles_and_url_forms() {
    for text in ["", " \t\n ", "#fragment"] {
        let url = CssUrl::new(text);
        assert_eq!(url.function(), CssUrlFunction::Url);
        assert_eq!(url.as_str(), text);
        assert!(url.modifiers().is_empty());
        let src = CssUrl::from_parts(
            CssUrlFunction::Src,
            text,
            vec![
                CssUrlModifier::Ident(CssIdent::try_new("cors").unwrap()),
                CssUrlModifier::Ident(CssIdent::try_new("integrity").unwrap()),
            ],
        );
        let import = CssImportUrl::new(src.clone());
        let font = CssFontFaceUrlSource::new(src.clone(), None, Vec::new());
        assert_eq!(import.url().function(), CssUrlFunction::Src);
        assert_eq!(font.url().function(), CssUrlFunction::Src);
        assert_eq!(import.url().as_str(), text);
        assert_eq!(font.url().as_str(), text);
        for actual in [import.url(), font.url()] {
            assert!(
                matches!(actual.modifiers(), [CssUrlModifier::Ident(first), CssUrlModifier::Ident(second)]
                if first.as_str() == "cors" && second.as_str() == "integrity")
            );
        }
        assert_eq!(CssImportString::new(text).as_str(), text);
        assert_eq!(font.format(), None);
        assert!(font.tech().is_empty());
        assert_ne!(url, src);
    }
}

#[test]
fn single_font_format_distinguishes_keyword_string_and_omission() {
    let keyword = CssFontFormat::Keyword(CssFontFormatHint::Woff2);
    let string = CssFontFormat::String(CssFontFormatString::new("WoFf2"));
    assert_ne!(keyword, string);
    assert_eq!(keyword.recognized_format(), Some(CssFontFormatHint::Woff2));
    assert_eq!(string.recognized_format(), Some(CssFontFormatHint::Woff2));
    for text in ["", "zebra", " woff2 ", "WoFf2"] {
        let source = CssFontFaceUrlSource::new(
            CssUrl::new(""),
            Some(CssFontFormat::String(CssFontFormatString::new(text))),
            Vec::new(),
        );
        let Some(CssFontFormat::String(value)) = source.format() else {
            panic!("present string argument");
        };
        assert_eq!(value.as_str(), text);
        assert_ne!(
            source,
            CssFontFaceUrlSource::new(CssUrl::new(""), None, Vec::new())
        );
    }
}

#[test]
fn legacy_variation_strings_derive_requirements_without_changing_authored_technologies() {
    for (text, format) in [
        ("WoFf2-VaRiAtIoNs", CssFontFormatHint::Woff2),
        ("woff-variations", CssFontFormatHint::Woff),
        ("truetype-variations", CssFontFormatHint::TrueType),
        ("opentype-variations", CssFontFormatHint::OpenType),
    ] {
        let source = CssFontFaceUrlSource::new(
            CssUrl::new("font"),
            Some(CssFontFormat::String(CssFontFormatString::new(text))),
            vec![
                CssFontTechHint::Palettes,
                CssFontTechHint::Palettes,
                CssFontTechHint::ColorCOLRv1,
            ],
        );
        assert_eq!(
            source.format().and_then(CssFontFormat::recognized_format),
            Some(format)
        );
        assert_eq!(
            source.tech(),
            [
                CssFontTechHint::Palettes,
                CssFontTechHint::Palettes,
                CssFontTechHint::ColorCOLRv1
            ]
        );
        assert_eq!(
            source.required_technologies().collect::<Vec<_>>(),
            [
                CssFontTechHint::Palettes,
                CssFontTechHint::ColorCOLRv1,
                CssFontTechHint::Variations
            ]
        );
        let Some(CssFontFormat::String(value)) = source.format() else {
            panic!("string format");
        };
        assert_eq!(value.as_str(), text);
    }
    let source = CssFontFaceUrlSource::new(
        CssUrl::new("font"),
        Some(CssFontFormat::String(CssFontFormatString::new(
            "woff2-variations",
        ))),
        vec![
            CssFontTechHint::Variations,
            CssFontTechHint::Palettes,
            CssFontTechHint::Variations,
        ],
    );
    assert_eq!(
        source.required_technologies().collect::<Vec<_>>(),
        [CssFontTechHint::Variations, CssFontTechHint::Palettes]
    );
}

#[test]
fn unitless_zero_has_no_fabricated_unit_and_explicit_zero_keeps_its_unit() {
    let zero = CssQueryLength::unitless_zero();
    assert_eq!(zero.value().value(), 0.0);
    assert_eq!(zero.unit(), None);
    for unit in [CssLengthUnit::Px, CssLengthUnit::Rem] {
        let explicit = CssQueryLength::try_new(0.0, unit).unwrap();
        assert_eq!(explicit.value().value(), 0.0);
        assert_eq!(explicit.unit(), Some(unit));
        assert_ne!(explicit, zero);
    }
}

#[test]
fn checked_grid_names_keep_empty_groups_order_and_repetitions() {
    assert!(CssGridLineNames::new(Vec::new()).names().is_empty());
    let names = CssGridLineNames::new(vec![
        CssGridLineName::try_new(CssIdent::try_new("start").unwrap()).unwrap(),
        CssGridLineName::try_new(CssIdent::try_new("end").unwrap()).unwrap(),
        CssGridLineName::try_new(CssIdent::try_new("start").unwrap()).unwrap(),
    ]);
    assert_eq!(
        names
            .names()
            .iter()
            .map(|name| name.ident().as_str())
            .collect::<Vec<_>>(),
        ["start", "end", "start"]
    );
}

#[test]
fn exact_numeric_tokens_preserve_kind_sign_spelling_and_programmatic_origin() {
    for (text, expected, kind, sign) in [
        ("+01", "+01", CssNumericTokenKind::Integer, true),
        ("-0.0", "-0.0", CssNumericTokenKind::Number, true),
        ("1e50", "1e50", CssNumericTokenKind::Number, false),
        ("1e-50%", "1e-50", CssNumericTokenKind::Number, false),
        ("+01.0PX", "+01.0", CssNumericTokenKind::Number, true),
    ] {
        let parsed = parse_component_values(text).unwrap();
        let programmatic = CssComponentValue::try_token(text).unwrap();
        assert!(matches!(
            parsed.items()[0].origin(),
            CssValueOrigin::Parsed(_)
        ));
        assert_eq!(programmatic.origin(), &CssValueOrigin::Programmatic);
        for component in [&parsed.items()[0], &programmatic] {
            let number = match component.view() {
                CssComponentValueRef::Token(
                    CssValueTokenRef::Number(number) | CssValueTokenRef::Percentage(number),
                ) => number,
                CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) => {
                    assert_eq!(unit, "PX");
                    number
                }
                _ => panic!("numeric token"),
            };
            assert_eq!(number.representation(), expected);
            assert_eq!(number.kind(), kind);
            assert_eq!(number.has_sign(), sign);
        }
        assert_eq!(parsed.serialize().unwrap().as_css(), text);
        assert_ne!(parsed.items()[0], programmatic);
    }
}
